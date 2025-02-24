//! A precompile to swap EVM tokens with native chain tokens using `Balanced` and `Inspect`
//! fungible interfaces.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::traits::Convert,
    sp_std::{marker::PhantomData, prelude::*},
    traits::{
        fungible::{Balanced, Inspect},
        tokens::{Fortitude, Precision, Preservation, Provenance},
    },
};
use pallet_evm::{
    ExitError, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use precompile_utils::{succeed, EvmDataWriter, EvmResult, PrecompileHandleExt};
use sp_core::{Get, H160, H256, U256};

use crate::{Config, EvmBalanceOf};

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
            .map_err(|error| match error {
                frame_support::sp_runtime::DispatchError::Token(
                    frame_support::sp_runtime::TokenError::BelowMinimum,
                ) => PrecompileFailure::Error {
                    exit_status: ExitError::OutOfFund,
                },
                _ => PrecompileFailure::Error {
                    exit_status: ExitError::Other("unable to deposit funds".into()),
                },
            })?;

        let credit = EvmSwapT::EvmToken::withdraw(
            &from,
            value,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Polite,
        )
        .unwrap();
        let _ = EvmSwapT::EvmToken::resolve(&EvmSwapT::BridgePotEvm::get(), credit);

        let credit = EvmSwapT::NativeToken::withdraw(
            &EvmSwapT::BridgePotNative::get(),
            estimated_swapped_balance,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Polite,
        )
        .unwrap();
        let _ = EvmSwapT::NativeToken::resolve(&to, credit);

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }
}
