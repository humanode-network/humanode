//! A precompile to swap EVM tokens with native chain tokens using `Balanced` and `Inspect`
//! fungible interfaces.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchError,
    sp_runtime::traits::Convert,
    sp_std::{marker::PhantomData, prelude::*},
    traits::{
        fungible::Inspect,
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

use crate::{balanced_transfer, Config, EvmBalanceOf};

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
pub struct EvmSwap<EvmSwapT, GasCost>(PhantomData<(EvmSwapT, GasCost)>)
where
    EvmSwapT: Config,
    EvmBalanceOf<EvmSwapT>: TryFrom<U256>,
    <EvmSwapT as Config>::EvmAccountId: From<H160>,
    <EvmSwapT as frame_system::Config>::AccountId: From<[u8; 32]>,
    GasCost: Get<u64>;

impl<EvmSwapT, GasCost> Precompile for EvmSwap<EvmSwapT, GasCost>
where
    EvmSwapT: Config,
    EvmBalanceOf<EvmSwapT>: TryFrom<U256>,
    <EvmSwapT as Config>::EvmAccountId: From<H160>,
    <EvmSwapT as frame_system::Config>::AccountId: From<[u8; 32]>,
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

impl<EvmSwapT, GasCost> EvmSwap<EvmSwapT, GasCost>
where
    EvmSwapT: Config,
    EvmBalanceOf<EvmSwapT>: TryFrom<U256>,
    <EvmSwapT as Config>::EvmAccountId: From<H160>,
    <EvmSwapT as frame_system::Config>::AccountId: From<[u8; 32]>,
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
        let value: EvmBalanceOf<EvmSwapT> =
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
        let to: EvmSwapT::AccountId = to.into();

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        // Here we must withdraw from self (i.e. from the precompile address, not from the caller
        // address), since the funds have already been transferred to us (precompile) as this point.
        let from: EvmSwapT::EvmAccountId = (*address).into();

        let estimated_swapped_balance = EvmSwapT::BalanceConverterEvmToNative::convert(value);

        EvmSwapT::NativeToken::can_deposit(&to, estimated_swapped_balance, Provenance::Extant)
            .into_result()
            .map_err(process_dispatch_error)?;

        balanced_transfer::<_, EvmSwapT::EvmToken>(
            &from,
            &EvmSwapT::BridgePotEvm::get(),
            value,
            Preservation::Expendable,
        )
        .map_err(process_dispatch_error)?;

        balanced_transfer::<_, EvmSwapT::NativeToken>(
            &EvmSwapT::BridgePotNative::get(),
            &to,
            estimated_swapped_balance,
            Preservation::Expendable,
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
        _ => PrecompileFailure::Error {
            exit_status: ExitError::Other("unable to swap funds".into()),
        },
    }
}
