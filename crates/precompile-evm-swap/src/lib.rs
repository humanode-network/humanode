//! A precompile to swap EVM tokens with native chain tokens using fungible interfaces.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchError,
    sp_runtime::traits::Convert,
    sp_std::{marker::PhantomData, prelude::*},
    traits::{
        fungible::{Inspect, Mutate},
        tokens::{Preservation, Provenance},
    },
};
use pallet_evm::{
    ExitError, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use precompile_utils::{
    keccak256, succeed, EvmDataWriter, EvmResult, LogExt, LogsBuilder, PrecompileHandleExt,
};
use sp_core::{Get, H160, H256, U256};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::NativeToken`] type.
pub type NativeBalanceOf<T> =
    <<T as Config>::NativeToken as Inspect<<T as Config>::AccountId>>::Balance;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::EvmToken`] type.
pub type EvmBalanceOf<T> =
    <<T as Config>::EvmToken as Inspect<<T as Config>::EvmAccountId>>::Balance;

/// The config for the swap logic.
pub trait Config {
    /// The native user account identifier type.
    type AccountId: From<[u8; 32]>;

    /// The EVM user account identifier type.
    type EvmAccountId: From<H160>;

    /// Native token.
    ///
    /// TODO(#1462): switch from `Mutate` to `Balanced` fungible interface.
    type NativeToken: Inspect<Self::AccountId> + Mutate<Self::AccountId>;

    /// EVM token.
    ///
    /// TODO(#1462): switch from `Mutate` to `Balanced` fungible interface.
    type EvmToken: Inspect<Self::EvmAccountId> + Mutate<Self::EvmAccountId>;

    /// The converter to determine how the balance amount should be converted from EVM
    /// to native token.
    type BalanceConverterEvmToNative: Convert<EvmBalanceOf<Self>, NativeBalanceOf<Self>>;

    /// The bridge pot native account.
    type BridgePotNative: Get<Self::AccountId>;

    /// The bridge pot EVM account.
    type BridgePotEvm: Get<Self::EvmAccountId>;
}

/// Solidity selector of the Swap log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_SWAP: [u8; 32] = keccak256!("Swap(address,bytes32,uint256)");

/// Possible actions for this interface.
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Swap EVM tokens to native tokens.
    Swap = "swap(bytes32)",
}

/// Exposes the EVM swap interface.
pub struct EvmSwap<ConfigT, GasCost>(PhantomData<(ConfigT, GasCost)>)
where
    ConfigT: Config,
    EvmBalanceOf<ConfigT>: TryFrom<U256>,
    ConfigT::EvmAccountId: From<H160>,
    ConfigT::AccountId: From<[u8; 32]>,
    GasCost: Get<u64>;

impl<ConfigT, GasCost> Precompile for EvmSwap<ConfigT, GasCost>
where
    ConfigT: Config,
    EvmBalanceOf<ConfigT>: TryFrom<U256>,
    ConfigT::EvmAccountId: From<H160>,
    ConfigT::AccountId: From<[u8; 32]>,
    GasCost: Get<u64>,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        handle.record_cost(GasCost::get())?;

        let selector = handle
            .read_selector()
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid function selector".into()),
            })?;

        match selector {
            Action::Swap => Self::swap(handle),
        }
    }
}

impl<ConfigT, GasCost> EvmSwap<ConfigT, GasCost>
where
    ConfigT: Config,
    EvmBalanceOf<ConfigT>: TryFrom<U256>,
    ConfigT::EvmAccountId: From<H160>,
    ConfigT::AccountId: From<[u8; 32]>,
    GasCost: Get<u64>,
{
    /// Swap EVM tokens to native tokens.
    fn swap(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        let fp_evm::Context {
            address,
            apparent_value: value,
            ..
        } = handle.context();

        let value_u256 = *value;
        let value: EvmBalanceOf<ConfigT> =
            (*value).try_into().map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?;

        input
            .expect_arguments(1)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly one argument is expected".into()),
            })?;

        let to_h256: H256 = input.read()?;
        let to: [u8; 32] = to_h256.into();
        let to: ConfigT::AccountId = to.into();

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        // Here we must withdraw from self (i.e. from the precompile address, not from the caller
        // address), since the funds have already been transferred to us (precompile) as this point.
        let from: ConfigT::EvmAccountId = (*address).into();

        let estimated_swapped_balance = ConfigT::BalanceConverterEvmToNative::convert(value);

        ConfigT::NativeToken::can_deposit(&to, estimated_swapped_balance, Provenance::Extant)
            .into_result()
            .map_err(process_dispatch_error)?;

        ConfigT::EvmToken::transfer(
            &from,
            &ConfigT::BridgePotEvm::get(),
            value,
            Preservation::Expendable,
        )
        .map_err(process_dispatch_error)?;

        ConfigT::NativeToken::transfer(
            &ConfigT::BridgePotNative::get(),
            &to,
            estimated_swapped_balance,
            // Bridge pot native account shouldn't be killed.
            Preservation::Preserve,
        )
        .map_err(process_dispatch_error)?;

        let logs_builder = LogsBuilder::new(handle.context().address);

        logs_builder
            .log3(
                SELECTOR_LOG_SWAP,
                handle.context().caller,
                to_h256,
                EvmDataWriter::new().write(value_u256).build(),
            )
            .record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }
}

/// A helper function to process dispatch related errors.
fn process_dispatch_error(error: DispatchError) -> PrecompileFailure {
    match error {
        DispatchError::Token(frame_support::sp_runtime::TokenError::FundsUnavailable) => {
            PrecompileFailure::Error {
                exit_status: ExitError::OutOfFund,
            }
        }
        DispatchError::Token(frame_support::sp_runtime::TokenError::BelowMinimum) => {
            PrecompileFailure::Error {
                exit_status: ExitError::Other(
                    "resulted balance is less than existential deposit".into(),
                ),
            }
        }
        DispatchError::Token(frame_support::sp_runtime::TokenError::NotExpendable) => {
            PrecompileFailure::Error {
                exit_status: ExitError::Other("account would be killed".into()),
            }
        }
        _ => PrecompileFailure::Error {
            exit_status: ExitError::Other("unable to execute swap".into()),
        },
    }
}
