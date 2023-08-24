//! A precompile to interact with `pallet_evm_balances` instances using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime,
    sp_std::{marker::PhantomData, prelude::*},
    storage::types::StorageDoubleMap,
    traits::{fungible::Inspect, tokens::currency::Currency, StorageInstance},
    Blake2_128Concat,
};
use pallet_evm::{
    ExitError, ExitRevert, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
    PrecompileResult,
};
use precompile_utils::{succeed, Address, Bytes, EvmDataWriter, EvmResult, PrecompileHandleExt};
use sp_core::{Get, H160, H256, U256};

/// Metadata of an ERC20 token.
pub trait Erc20Metadata {
    /// Returns the name of the token.
    fn name() -> &'static str;

    /// Returns the symbol of the token.
    fn symbol() -> &'static str;

    /// Returns the decimals places of the token.
    fn decimals() -> u8;
}

pub struct Approves;

impl StorageInstance for Approves {
    const STORAGE_PREFIX: &'static str = "Approves";

    fn pallet_prefix() -> &'static str {
        "Erc20InstanceEvmBalances"
    }
}

/// Storage type used to store approvals, since `pallet_balances` doesn't
/// handle this behavior.
/// (Owner => Allowed => Amount)
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
    Name = "name()",
    Symbol = "symbol()",
    Decimals = "decimals()",
    TotalSupply = "totalSupply()",
    BalanceOf = "balanceOf(address)",
    Allowance = "allowance(address,address)",
    Approve = "approve(address,uint256)",
}

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
    fn name(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let name: Bytes = Metadata::name().into();

        Ok(succeed(EvmDataWriter::new().write(name).build()))
    }

    fn symbol(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let symbol: Bytes = Metadata::symbol().into();

        Ok(succeed(EvmDataWriter::new().write(symbol).build()))
    }

    fn decimals(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let decimals: u8 = Metadata::decimals();

        Ok(succeed(EvmDataWriter::new().write(decimals).build()))
    }

    fn total_supply(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let total_supply: U256 =
            pallet_evm_balances::Pallet::<EvmBalancesT>::total_issuance().into();

        Ok(succeed(EvmDataWriter::new().write(total_supply).build()))
    }

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

    fn allowance(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        input
            .expect_arguments(2)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly two argument is expected".into()),
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

    fn approve(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        handle.record_cost(GasCost::get())?;

        let mut input = handle.read_input()?;

        let owner = handle.context().caller;
        let owner: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = owner.into();

        input
            .expect_arguments(2)
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("exactly two argument is expected".into()),
            })?;

        let spender: Address = input.read()?;
        let spender: H160 = spender.into();
        let spender: <EvmBalancesT as pallet_evm_balances::Config>::AccountId = spender.into();

        let amount: U256 = input.read()?;
        let amount: <EvmBalancesT as pallet_evm_balances::Config>::Balance =
            amount.try_into().map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?;

        let junk_data = input.read_till_end()?;
        if !junk_data.is_empty() {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("junk at the end of input".into()),
            });
        }

        ApprovesStorage::<EvmBalancesT>::insert(owner.clone(), spender.clone(), amount);

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }
}
