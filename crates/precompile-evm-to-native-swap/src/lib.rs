//! A precompile to swap EVM tokens to native chain tokens using fungible interfaces.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchError,
    sp_runtime::traits::Convert,
    sp_std::{marker::PhantomData, prelude::*},
    traits::{
        fungible::{Inspect, Mutate},
        tokens::{Preservation, Provenance},
    },
    weights::RuntimeDbWeight,
};
use pallet_evm::{
    ExitError, ExitRevert, GasWeightMapping, Log, Precompile, PrecompileFailure, PrecompileHandle,
    PrecompileOutput, PrecompileResult,
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
type NativeBalanceOf<T> =
    <<T as Config>::NativeToken as Inspect<<T as Config>::AccountId>>::Balance;

/// Utility alias for easy access to the [`Inspect::Balance`] of the [`Config::EvmToken`] type.
type EvmBalanceOf<T> = <<T as Config>::EvmToken as Inspect<<T as Config>::EvmAccountId>>::Balance;

/// The config for EVM to native swap logic.
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

    /// The weight of the runtime database operations, used to price the swap.
    type DbWeight: Get<RuntimeDbWeight>;
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

/// Exposes the swap interface.
///
/// The calls are priced through the `GasWeightMapping`, see [`EvmToNativeSwap::gas_cost`].
pub struct EvmToNativeSwap<C, GasWeightMapping>(PhantomData<(C, GasWeightMapping)>)
where
    C: Config,
    EvmBalanceOf<C>: TryFrom<U256>,
    C::EvmAccountId: From<H160>,
    C::AccountId: From<[u8; 32]>;

impl<C, G> Precompile for EvmToNativeSwap<C, G>
where
    C: Config,
    EvmBalanceOf<C>: TryFrom<U256>,
    C::EvmAccountId: From<H160>,
    C::AccountId: From<[u8; 32]>,
    G: GasWeightMapping,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        ensure_not_delegate_call(handle)?;

        handle.record_cost(Self::gas_cost())?;

        let selector = handle
            .read_selector()
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid function selector".into()),
            })?;

        match selector {
            Action::Swap => {
                ensure_not_static(handle)?;
                Self::swap(handle)
            }
        }
    }
}

impl<C, G> EvmToNativeSwap<C, G>
where
    C: Config,
    EvmBalanceOf<C>: TryFrom<U256>,
    C::EvmAccountId: From<H160>,
    C::AccountId: From<[u8; 32]>,
    G: GasWeightMapping,
{
    /// The gas cost of a call to the precompile.
    ///
    /// The swap reads and writes the balances of the two accounts on the EVM side and of the two
    /// accounts on the native side, priced from the runtime DB weights through the gas-to-weight
    /// mapping, the same way the EVM opcodes are priced.
    pub fn gas_cost() -> u64 {
        G::weight_to_gas(C::DbWeight::get().reads_writes(4, 4))
    }

    /// Swap EVM tokens to native chain tokens.
    fn swap(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        let fp_evm::Context {
            address,
            apparent_value: value,
            ..
        } = handle.context();

        let value_u256 = *value;
        let value: EvmBalanceOf<C> = (*value).try_into().map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::Other("value is out of bounds".into()),
        })?;

        input
            .expect_arguments(1)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly one argument is expected".into()),
            })?;

        let to_h256: H256 = input.read()?;
        let to: [u8; 32] = to_h256.into();
        let to: C::AccountId = to.into();

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        // Here we must withdraw from self (i.e. from the precompile address, not from the caller
        // address), since the funds have already been transferred to us (precompile) as this point.
        let from: C::EvmAccountId = (*address).into();

        let estimated_swapped_balance = C::BalanceConverterEvmToNative::convert(value);

        C::NativeToken::can_deposit(&to, estimated_swapped_balance, Provenance::Extant)
            .into_result()
            .map_err(process_dispatch_error)?;

        C::EvmToken::transfer(
            &from,
            &C::BridgePotEvm::get(),
            value,
            Preservation::Expendable,
        )
        .map_err(process_dispatch_error)?;

        C::NativeToken::transfer(
            &C::BridgePotNative::get(),
            &to,
            estimated_swapped_balance,
            // Bridge pot native account shouldn't be killed.
            Preservation::Preserve,
        )
        .map_err(process_dispatch_error)?;

        let log = LogsBuilder::new(handle.context().address).log3(
            SELECTOR_LOG_SWAP,
            handle.context().caller,
            to_h256,
            EvmDataWriter::new().write(value_u256).build(),
        );
        record_log_cost(handle, &log)?;
        log.record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }
}

/// The gas costs of a log, as the EVM charges for the `LOG` opcodes.
///
/// Ref: <https://github.com/rust-blockchain/evm/blob/master/gasometer/src/consts.rs>.
mod log_gas {
    /// The base cost of a log.
    pub const G_LOG: u64 = 375;
    /// The cost of a log data byte.
    pub const G_LOGDATA: u64 = 8;
    /// The cost of a log topic.
    pub const G_LOGTOPIC: u64 = 375;
}

/// Records the gas cost of the `log` the same way the EVM charges for the `LOG` opcodes.
///
/// The precompile handle records the logs without charging for them, so the cost has to be
/// recorded explicitly.
fn record_log_cost(handle: &mut impl PrecompileHandle, log: &Log) -> EvmResult {
    let cost = log_gas::G_LOG
        .saturating_add(log_gas::G_LOGTOPIC.saturating_mul(log.topics.len() as u64))
        .saturating_add(log_gas::G_LOGDATA.saturating_mul(log.data.len() as u64));
    handle.record_cost(cost)?;
    Ok(())
}

/// Rejects the call when it is made via `DELEGATECALL` or `CALLCODE`.
///
/// Under those the context address belongs to the calling frame and differs from the precompile
/// code address. The runtime precompile set guards against this too; this check keeps
/// the precompile safe on its own, regardless of how it is wired.
fn ensure_not_delegate_call(handle: &impl PrecompileHandle) -> EvmResult {
    if handle.code_address() != handle.context().address {
        return Err(PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: b"cannot be called with DELEGATECALL or CALLCODE".to_vec(),
        });
    }
    Ok(())
}

/// Rejects the call when it is made in a static context (i.e. via `STATICCALL`).
///
/// The swap changes state, so it must not be reachable from a context the EVM promises is
/// read-only.
fn ensure_not_static(handle: &impl PrecompileHandle) -> EvmResult {
    if handle.is_static() {
        return Err(PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: b"cannot be called with STATICCALL".to_vec(),
        });
    }
    Ok(())
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
