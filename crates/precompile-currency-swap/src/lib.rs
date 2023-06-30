//! A precompile to swap EVM tokens with native chain tokens.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::tokens::currency::Currency;
use pallet_evm::{
    ExitError, ExitRevert, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
    PrecompileResult,
};
use precompile_utils::{succeed, EvmDataWriter, EvmResult, PrecompileHandleExt};
use sp_core::{Get, H160, H256, U256};
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Possible actions for this interface.
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Swap EVM tokens to native tokens.
    Swap = "swap(bytes32)",
}

/// Exposes the currency swap interface to EVM.
pub struct CurrencySwap<CurrencySwapT, AccountIdFrom, AccountIdTo, GasCost>(
    PhantomData<(CurrencySwapT, AccountIdFrom, AccountIdTo, GasCost)>,
)
where
    AccountIdFrom: From<H160>,
    AccountIdTo: From<[u8; 32]>,
    CurrencySwapT: primitives_currency_swap::CurrencySwap<AccountIdFrom, AccountIdTo>,
    FromBalanceFor<CurrencySwapT, AccountIdFrom, AccountIdTo>: TryFrom<U256>,
    GasCost: Get<u64>;

impl<CurrencySwapT, AccountIdFrom, AccountIdTo, GasCost> Precompile
    for CurrencySwap<CurrencySwapT, AccountIdFrom, AccountIdTo, GasCost>
where
    AccountIdFrom: From<H160>,
    AccountIdTo: From<[u8; 32]>,
    CurrencySwapT: primitives_currency_swap::CurrencySwap<AccountIdFrom, AccountIdTo>,
    FromBalanceFor<CurrencySwapT, AccountIdFrom, AccountIdTo>: TryFrom<U256>,
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

/// Utility alias for easy access to [`CurrencySwap::From::Balance`] type.
type FromBalanceFor<T, AccountIdFrom, AccountIdTo> =
    <FromCurrencyFor<T, AccountIdFrom, AccountIdTo> as Currency<AccountIdFrom>>::Balance;

/// Utility alias for easy access to [`CurrencySwap::From`] type.
type FromCurrencyFor<T, AccountIdFrom, AccountIdTo> =
    <T as primitives_currency_swap::CurrencySwap<AccountIdFrom, AccountIdTo>>::From;

impl<CurrencySwapT, AccountIdFrom, AccountIdTo, GasCost>
    CurrencySwap<CurrencySwapT, AccountIdFrom, AccountIdTo, GasCost>
where
    AccountIdFrom: From<H160>,
    AccountIdTo: From<[u8; 32]>,
    CurrencySwapT: primitives_currency_swap::CurrencySwap<AccountIdFrom, AccountIdTo>,
    FromBalanceFor<CurrencySwapT, AccountIdFrom, AccountIdTo>: TryFrom<U256>,
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

        // Here we must withdraw from self (i.e. from the precompile address, not from the caller
        // address), since the funds have already been transferred to us (precompile) as this point.
        let from: AccountIdFrom = (*address).into();

        let value: FromBalanceFor<CurrencySwapT, AccountIdFrom, AccountIdTo> =
            (*value).try_into().map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?;

        input
            .expect_arguments(1)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly one argument is expected".into()),
            })?;

        let to: H256 = input.read()?;
        let to: [u8; 32] = to.into();
        let to: AccountIdTo = to.into();

        let imbalance = CurrencySwapT::From::withdraw(
            &from,
            value,
            frame_support::traits::WithdrawReasons::TRANSFER,
            frame_support::traits::ExistenceRequirement::AllowDeath,
        )
        .map_err(|error| match error {
            sp_runtime::DispatchError::Token(sp_runtime::TokenError::NoFunds) => {
                PrecompileFailure::Error {
                    exit_status: ExitError::OutOfFund,
                }
            }
            _ => PrecompileFailure::Error {
                exit_status: ExitError::Other("unable to withdraw funds".into()),
            },
        })?;

        let imbalance = CurrencySwapT::swap(imbalance).map_err(|_| PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: "unable to swap the currency".into(),
        })?;

        CurrencySwapT::To::resolve_creating(&to, imbalance);

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }
}
