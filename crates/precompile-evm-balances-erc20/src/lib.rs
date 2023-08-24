//! A precompile to interact with `pallet_evm_balances` instances using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime,
    sp_std::{marker::PhantomData, prelude::*},
    storage::types::StorageDoubleMap,
    traits::{fungible::Inspect, tokens::currency::Currency},
};
use pallet_evm::{
    ExitError, ExitRevert, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
    PrecompileResult,
};
use precompile_utils::{succeed, Address, EvmDataWriter, EvmResult, PrecompileHandleExt};
use sp_core::{Get, H160, H256, U256};

/// Possible actions for this interface.
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
    TotalSupply = "totalSupply()",
    BalanceOf = "balanceOf(address)",
}

pub struct EvmBalancesErc20<Runtime, GasCost>(PhantomData<(Runtime, GasCost)>)
where
    GasCost: Get<u64>;

impl<EvmBalancesT, GasCost> Precompile for EvmBalancesErc20<EvmBalancesT, GasCost>
where
    EvmBalancesT: pallet_evm_balances::Config,
    <EvmBalancesT as pallet_evm_balances::Config>::Balance: Into<U256>,
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
            Action::TotalSupply => Self::total_supply(handle),
            Action::BalanceOf => Self::balance_of(handle),
        }
    }
}

impl<EvmBalancesT, GasCost> EvmBalancesErc20<EvmBalancesT, GasCost>
where
    EvmBalancesT: pallet_evm_balances::Config,
    <EvmBalancesT as pallet_evm_balances::Config>::Balance: Into<U256>,
    <EvmBalancesT as pallet_evm_balances::Config>::AccountId: From<H160>,
    GasCost: Get<u64>,
{
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

        let address: Address = input.read()?;
        let address: H160 = address.into();

        let total_balance: U256 =
            pallet_evm_balances::Pallet::<EvmBalancesT>::total_balance(&address.into()).into();
        Ok(succeed(EvmDataWriter::new().write(total_balance).build()))
    }
}
