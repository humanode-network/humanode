//! A precompile to interact with `pallet_evm_balances` instances using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::{self, traits::CheckedSub},
    sp_std::{marker::PhantomData, prelude::*},
    storage::types::StorageDoubleMap,
    traits::{tokens::currency::Currency, StorageInstance},
    Blake2_128Concat,
};
use pallet_evm::{
    ExitError, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use precompile_utils::{
    keccak256, revert, succeed, Address, Bytes, EvmDataWriter, EvmResult, LogExt, LogsBuilder,
    PrecompileHandleExt,
};
use sp_core::{Get, H160, U256};

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

/// Metadata of an ERC20 token.
pub trait Erc20Metadata {
    /// Returns the name of the token.
    fn name() -> &'static str;

    /// Returns the symbol of the token.
    fn symbol() -> &'static str;

    /// Returns the decimals places of the token.
    fn decimals() -> u8;
}

/// Approves storage instance to manage the allowance related data.
pub struct Approves;

impl StorageInstance for Approves {
    const STORAGE_PREFIX: &'static str = "Approves";

    fn pallet_prefix() -> &'static str {
        "EvmBalancesErc20Approves"
    }
}

/// Storage type used to store approvals, since `pallet_evm_balances` doesn't handle this behavior.
/// (Owner => Allowed => Amount).
pub type ApprovesStorage<Runtime> = StorageDoubleMap<
    Approves,
    Blake2_128Concat,
    <Runtime as pallet_evm_balances::Config>::AccountId,
    Blake2_128Concat,
    <Runtime as pallet_evm_balances::Config>::AccountId,
    <Runtime as pallet_evm_balances::Config>::Balance,
>;

/// Possible actions for this interface.
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Returns the name of the token.
    Name = "name()",
    /// Returns the symbol of the token.
    Symbol = "symbol()",
    /// Returns the decimals places of the token.
    Decimals = "decimals()",
    /// Returns the amount of tokens in existence.
    TotalSupply = "totalSupply()",
    /// Returns the amount of tokens owned by provided account.
    BalanceOf = "balanceOf(address)",
    /// Returns the remaining number of tokens that spender will be allowed to spend on behalf of
    /// owner through transferFrom. This is zero by default.
    Allowance = "allowance(address,address)",
    /// Sets amount as the allowance of spender over the caller’s tokens.
    Approve = "approve(address,uint256)",
    /// Moves amount tokens from the caller’s account to recipient.
    Transfer = "transfer(address,uint256)",
    /// Moves amount tokens from sender to recipient using the allowance mechanism,
    /// amount is then deducted from the caller’s allowance.
    TransferFrom = "transferFrom(address,address,uint256)",
}

/// Precompile exposing a `pallet_evm_balance` as an ERC20.
pub struct EvmBalancesErc20<Runtime, Metadata, GasCost>(PhantomData<(Runtime, Metadata, GasCost)>)
where
    GasCost: Get<u64>;

impl<EvmBalancesT, Metadata, GasCost> Precompile
    for EvmBalancesErc20<EvmBalancesT, Metadata, GasCost>
where
    Metadata: Erc20Metadata,
    EvmBalancesT: pallet_evm_balances::Config,
    <EvmBalancesT as pallet_evm_balances::Config>::Balance: Into<U256> + TryFrom<U256>,
    <EvmBalancesT as pallet_evm_balances::Config>::AccountId: From<H160>,
    GasCost: Get<u64>,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let selector = handle
            .read_selector()
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid function selector".into()),
            })?;

        match selector {
            Action::Name => Self::name(handle),
            Action::Symbol => Self::symbol(handle),
            Action::Decimals => Self::decimals(handle),
            Action::TotalSupply => Self::total_supply(handle),
            Action::BalanceOf => Self::balance_of(handle),
            Action::Allowance => Self::allowance(handle),
            Action::Approve => Self::approve(handle),
            Action::Transfer => Self::transfer(handle),
            Action::TransferFrom => Self::transfer_from(handle),
        }
    }
}

impl<EvmBalancesT, Metadata, GasCost> EvmBalancesErc20<EvmBalancesT, Metadata, GasCost>
where
    Metadata: Erc20Metadata,
    EvmBalancesT: pallet_evm_balances::Config,
    <EvmBalancesT as pallet_evm_balances::Config>::Balance: Into<U256> + TryFrom<U256>,
    <EvmBalancesT as pallet_evm_balances::Config>::AccountId: From<H160>,
    GasCost: Get<u64>,
{
    /// Returns the name of the token.
    fn name(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let name: Bytes = Metadata::name().into();

        Ok(succeed(EvmDataWriter::new().write(name).build()))
    }

    /// Returns the symbol of the token.
    fn symbol(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let symbol: Bytes = Metadata::symbol().into();

        Ok(succeed(EvmDataWriter::new().write(symbol).build()))
    }

    /// Returns the decimals places of the token.
    fn decimals(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let decimals: u8 = Metadata::decimals();

        Ok(succeed(EvmDataWriter::new().write(decimals).build()))
    }

    /// Returns the amount of tokens in existence.
    fn total_supply(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let total_supply: U256 =
            pallet_evm_balances::Pallet::<EvmBalancesT>::total_issuance().into();

        Ok(succeed(EvmDataWriter::new().write(total_supply).build()))
    }

    /// Returns the amount of tokens owned by provided account.
    fn balance_of(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        input
            .expect_arguments(1)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly one argument is expected".into()),
            })?;

        let owner: Address = input.read()?;
        let owner: H160 = owner.into();

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        let total_balance: U256 =
            pallet_evm_balances::Pallet::<EvmBalancesT>::total_balance(&owner.into()).into();

        Ok(succeed(EvmDataWriter::new().write(total_balance).build()))
    }

    /// Returns the remaining number of tokens that spender will be allowed to spend on behalf of
    /// owner through transferFrom. This is zero by default.
    fn allowance(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        input
            .expect_arguments(2)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly two arguments are expected".into()),
            })?;

        let owner: Address = input.read()?;
        let owner: H160 = owner.into();
        let owner: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = owner.into();

        let spender: Address = input.read()?;
        let spender: H160 = spender.into();
        let spender: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = spender.into();

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        Ok(succeed(
            EvmDataWriter::new()
                .write(
                    ApprovesStorage::<EvmBalancesT>::get(owner, spender)
                        .unwrap_or_default()
                        .into(),
                )
                .build(),
        ))
    }

    /// Sets amount as the allowance of spender over the caller’s tokens.
    fn approve(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        handle.record_cost(GasCost::get())?;

        let mut input = handle.read_input()?;

        let owner = handle.context().caller;
        let owner: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = owner.into();

        input
            .expect_arguments(2)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly two arguments are expected".into()),
            })?;

        let spender: Address = input.read()?;
        let spender_h160: H160 = spender.into();
        let spender: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = spender_h160.into();

        let amount_u256: U256 = input.read()?;
        let amount: <EvmBalancesT as pallet_evm_balances::Config>::Balance = amount_u256
            .try_into()
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?;

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        ApprovesStorage::<EvmBalancesT>::insert(owner, spender, amount);

        let logs_builder = LogsBuilder::new(handle.context().address);

        logs_builder
            .log3(
                SELECTOR_LOG_APPROVAL,
                handle.context().caller,
                spender_h160,
                EvmDataWriter::new().write(amount_u256).build(),
            )
            .record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    /// Moves amount tokens from the caller’s account to recipient.
    fn transfer(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        handle.record_cost(GasCost::get())?;

        let mut input = handle.read_input()?;

        let from = handle.context().caller;
        let from: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = from.into();

        input
            .expect_arguments(2)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly two arguments are expected".into()),
            })?;

        let to: Address = input.read()?;
        let to_h160: H160 = to.into();
        let to: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = to_h160.into();

        let amount_u256: U256 = input.read()?;
        let amount: <EvmBalancesT as pallet_evm_balances::Config>::Balance = amount_u256
            .try_into()
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?;

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        pallet_evm_balances::Pallet::<EvmBalancesT>::transfer(
            &from,
            &to,
            amount,
            frame_support::traits::ExistenceRequirement::AllowDeath,
        )
        .map_err(|error| match error {
            sp_runtime::DispatchError::Token(sp_runtime::TokenError::NoFunds) => {
                PrecompileFailure::Error {
                    exit_status: ExitError::OutOfFund,
                }
            }
            _ => PrecompileFailure::Error {
                exit_status: ExitError::Other("unable to transfer funds".into()),
            },
        })?;

        let logs_builder = LogsBuilder::new(handle.context().address);

        logs_builder
            .log3(
                SELECTOR_LOG_TRANSFER,
                handle.context().caller,
                to_h160,
                EvmDataWriter::new().write(amount_u256).build(),
            )
            .record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    /// Moves amount tokens from sender to recipient using the allowance mechanism,
    /// amount is then deducted from the caller’s allowance.
    fn transfer_from(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        handle.record_cost(GasCost::get())?;

        let mut input = handle.read_input()?;

        let caller = handle.context().caller;
        let caller: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = caller.into();

        input
            .expect_arguments(3)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly three argument are expected".into()),
            })?;

        let from: Address = input.read()?;
        let from_h160: H160 = from.into();
        let from: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = from_h160.into();

        let to: Address = input.read()?;
        let to_h160: H160 = to.into();
        let to: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = to_h160.into();

        let amount_u256: U256 = input.read()?;
        let amount: <EvmBalancesT as pallet_evm_balances::Config>::Balance = amount_u256
            .try_into()
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?;

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        // If caller is "from", it can spend as much as it wants.
        if caller != from {
            ApprovesStorage::<EvmBalancesT>::mutate(from.clone(), caller, |entry| {
                // Get current allowed value, exit if None.
                let allowed = entry.ok_or(revert("spender not allowed"))?;

                // Remove "value" from allowed, exit if underflow.
                let allowed = allowed
                    .checked_sub(&amount)
                    .ok_or_else(|| revert("trying to spend more than allowed"))?;

                // Update allowed value.
                *entry = Some(allowed);

                EvmResult::Ok(())
            })?;
        }

        pallet_evm_balances::Pallet::<EvmBalancesT>::transfer(
            &from,
            &to,
            amount,
            frame_support::traits::ExistenceRequirement::AllowDeath,
        )
        .map_err(|error| match error {
            sp_runtime::DispatchError::Token(sp_runtime::TokenError::NoFunds) => {
                PrecompileFailure::Error {
                    exit_status: ExitError::OutOfFund,
                }
            }
            _ => PrecompileFailure::Error {
                exit_status: ExitError::Other("unable to transfer funds".into()),
            },
        })?;

        let logs_builder = LogsBuilder::new(handle.context().address);

        logs_builder
            .log4(
                SELECTOR_LOG_TRANSFER,
                handle.context().caller,
                from_h160,
                to_h160,
                EvmDataWriter::new().write(amount_u256).build(),
            )
            .record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }
}
