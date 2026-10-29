//! A precompile to interact with currency instance using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    sp_runtime::{self, DispatchError},
    sp_std::{marker::PhantomData, prelude::*},
    traits::Currency,
};
use pallet_erc20_support::Metadata;
use pallet_evm::{
    ExitError, ExitRevert, GasWeightMapping, Log, Precompile, PrecompileFailure, PrecompileHandle,
    PrecompileOutput, PrecompileResult,
};
use precompile_utils::{
    keccak256, succeed, Address, Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, LogExt,
    LogsBuilder, PrecompileHandleExt,
};
use sp_core::{Get, H160, U256};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

/// Solidity selector of the Deposit log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_DEPOSIT: [u8; 32] = keccak256!("Deposit(address,uint256)");

/// Solidity selector of the Withdraw log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_WITHDRAWAL: [u8; 32] = keccak256!("Withdrawal(address,uint256)");

/// Utility alias for easy access to the [`pallet_erc20_support::Config::AccountId`].
type AccountIdOf<T> = <T as pallet_erc20_support::Config>::AccountId;

/// Utility alias for easy access to the [`Currency::Balance`] of the [`pallet_erc20_support::Config::Currency`] type.
type BalanceOf<T> =
    <<T as pallet_erc20_support::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

/// Utility alias for easy access to the [`pallet_erc20_support::Config::Allowance`].
type AllowanceOf<T> = <T as pallet_erc20_support::Config>::Allowance;

/// Possible actions for this interface.
#[precompile_utils::generate_function_selector]
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Simulate deposit logic as IWETH-like contract.
    /// Returns funds to sender as this precompile tokens and the native tokens are the same.
    Deposit = "deposit()",
    /// Simulate withdraw logic as IWETH-like contract.
    /// Do nothing.
    Withdraw = "withdraw(uint256)",
}

/// Precompile exposing currency instance as ERC20.
///
/// The calls are priced through the `GasWeightMapping`, see [`NativeCurrency::gas_cost`].
pub struct NativeCurrency<Erc20SupportT, GasWeightMapping>(
    PhantomData<(Erc20SupportT, GasWeightMapping)>,
);

impl<Erc20SupportT, G> Precompile for NativeCurrency<Erc20SupportT, G>
where
    Erc20SupportT: pallet_erc20_support::Config,
    AccountIdOf<Erc20SupportT>: From<H160>,
    BalanceOf<Erc20SupportT>: Into<U256> + TryFrom<U256>,
    AllowanceOf<Erc20SupportT>: TryFrom<U256> + EvmData,
    G: GasWeightMapping,
{
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        ensure_not_delegate_call(handle)?;

        let selector = handle
            .read_selector()
            .map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("invalid function selector".into()),
            })?;

        // Enforces the Solidity-level semantics of each function: the view functions are the only
        // ones callable via `STATICCALL`, and only `deposit` accepts value.
        match selector {
            Action::Name
            | Action::Symbol
            | Action::Decimals
            | Action::TotalSupply
            | Action::BalanceOf
            | Action::Allowance => ensure_no_value(handle)?,
            Action::Approve | Action::Transfer | Action::TransferFrom | Action::Withdraw => {
                ensure_not_static(handle)?;
                ensure_no_value(handle)?;
            }
            Action::Deposit => ensure_not_static(handle)?,
        }

        handle.record_cost(Self::gas_cost(selector))?;

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
            Action::Deposit => Self::deposit(handle),
            Action::Withdraw => Self::withdraw(handle),
        }
    }
}

impl<Erc20SupportT, G> NativeCurrency<Erc20SupportT, G>
where
    Erc20SupportT: pallet_erc20_support::Config,
    AccountIdOf<Erc20SupportT>: From<H160>,
    BalanceOf<Erc20SupportT>: Into<U256> + TryFrom<U256>,
    AllowanceOf<Erc20SupportT>: TryFrom<U256> + EvmData,
    G: GasWeightMapping,
{
    /// The gas cost of a call to the given function.
    ///
    /// Each function is priced from the runtime DB weights of the storage reads and writes it
    /// performs, through the gas-to-weight mapping, the same way the EVM opcodes are priced.
    pub fn gas_cost(action: Action) -> u64 {
        let (reads, writes) = Self::db_ops(action);
        G::weight_to_gas(
            <Erc20SupportT as frame_system::Config>::DbWeight::get().reads_writes(reads, writes),
        )
    }

    /// The number of storage reads and writes the given function performs.
    fn db_ops(action: Action) -> (u64, u64) {
        match action {
            // The metadata is static.
            Action::Name | Action::Symbol | Action::Decimals => (0, 0),
            // A single read: the total issuance, the owner account, the allowance, and the caller
            // account respectively.
            Action::TotalSupply | Action::BalanceOf | Action::Allowance | Action::Withdraw => {
                (1, 0)
            }
            // The allowance write.
            Action::Approve => (0, 1),
            // The reads and writes of two accounts: the caller and the recipient, and
            // the precompile and the caller respectively.
            Action::Transfer | Action::Deposit => (2, 2),
            // The allowance read and write, plus the reads and writes of the sender and
            // recipient accounts.
            Action::TransferFrom => (3, 3),
        }
    }

    /// Returns the name of the token.
    fn name(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let name: Bytes = Erc20SupportT::Metadata::name().into();

        Ok(succeed(EvmDataWriter::new().write(name).build()))
    }

    /// Returns the symbol of the token.
    fn symbol(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let symbol: Bytes = Erc20SupportT::Metadata::symbol().into();

        Ok(succeed(EvmDataWriter::new().write(symbol).build()))
    }

    /// Returns the decimals places of the token.
    fn decimals(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let decimals: u8 = Erc20SupportT::Metadata::decimals();

        Ok(succeed(EvmDataWriter::new().write(decimals).build()))
    }

    /// Returns the amount of tokens in existence.
    fn total_supply(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let total_supply: U256 =
            pallet_erc20_support::Pallet::<Erc20SupportT>::total_supply().into();

        Ok(succeed(EvmDataWriter::new().write(total_supply).build()))
    }

    /// Returns the amount of tokens owned by provided account.
    fn balance_of(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;
        check_input(&mut input, 1)?;

        let owner: Address = input.read()?;
        let owner: H160 = owner.into();

        check_input_end(&mut input)?;

        let total_balance: U256 =
            pallet_erc20_support::Pallet::<Erc20SupportT>::balance_of(&owner.into()).into();

        Ok(succeed(EvmDataWriter::new().write(total_balance).build()))
    }

    /// Returns the remaining number of tokens that spender will be allowed to spend on behalf of
    /// owner through transferFrom. This is zero by default.
    fn allowance(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        check_input(&mut input, 2)?;

        let owner: Address = input.read()?;
        let owner: H160 = owner.into();

        let spender: Address = input.read()?;
        let spender: H160 = spender.into();

        check_input_end(&mut input)?;

        Ok(succeed(
            EvmDataWriter::new()
                .write(pallet_erc20_support::Pallet::<Erc20SupportT>::allowance(
                    &owner.into(),
                    &spender.into(),
                ))
                .build(),
        ))
    }

    /// Sets amount as the allowance of spender over the caller’s tokens.
    fn approve(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        let owner = handle.context().caller;

        check_input(&mut input, 2)?;

        let spender: Address = input.read()?;
        let spender: H160 = spender.into();

        let amount: U256 = input.read()?;

        check_input_end(&mut input)?;

        pallet_erc20_support::Pallet::<Erc20SupportT>::approve(
            owner.into(),
            spender.into(),
            amount.try_into().map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("allowance is out of bounds".into()),
            })?,
        );

        let log = LogsBuilder::new(handle.context().address).log3(
            SELECTOR_LOG_APPROVAL,
            handle.context().caller,
            spender,
            EvmDataWriter::new().write(amount).build(),
        );
        record_log_cost(handle, &log)?;
        log.record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    /// Moves amount tokens from the caller’s account to recipient.
    fn transfer(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        let caller = handle.context().caller;

        check_input(&mut input, 2)?;

        let recipient: Address = input.read()?;
        let recipient: H160 = recipient.into();

        let amount: U256 = input.read()?;

        check_input_end(&mut input)?;

        pallet_erc20_support::Pallet::<Erc20SupportT>::transfer(
            caller.into(),
            recipient.into(),
            amount.try_into().map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?,
        )
        .map_err(process_dispatch_error::<Erc20SupportT>)?;

        let log = LogsBuilder::new(handle.context().address).log3(
            SELECTOR_LOG_TRANSFER,
            caller,
            recipient,
            EvmDataWriter::new().write(amount).build(),
        );
        record_log_cost(handle, &log)?;
        log.record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    /// Moves amount tokens from sender to recipient using the allowance mechanism,
    /// amount is then deducted from the caller’s allowance.
    fn transfer_from(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        let caller = handle.context().caller;

        check_input(&mut input, 3)?;

        let sender: Address = input.read()?;
        let sender: H160 = sender.into();

        let recipient: Address = input.read()?;
        let recipient: H160 = recipient.into();

        let amount: U256 = input.read()?;

        check_input_end(&mut input)?;

        pallet_erc20_support::Pallet::<Erc20SupportT>::transfer_from(
            caller.into(),
            sender.into(),
            recipient.into(),
            amount.try_into().map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?,
        )
        .map_err(process_dispatch_error::<Erc20SupportT>)?;

        let log = LogsBuilder::new(handle.context().address).log3(
            SELECTOR_LOG_TRANSFER,
            sender,
            recipient,
            EvmDataWriter::new().write(amount).build(),
        );
        record_log_cost(handle, &log)?;
        log.record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    /// Simulate deposit logic as IWETH-like contract.
    /// Returns funds to sender as this precompile tokens and the native tokens are the same.
    fn deposit(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        let fp_evm::Context {
            address,
            caller,
            apparent_value: value,
            ..
        } = handle.context();

        check_input(&mut input, 0)?;
        check_input_end(&mut input)?;

        if value == &U256::from(0u32) {
            return Err(PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "deposited amount must be non-zero".into(),
            });
        }

        pallet_erc20_support::Pallet::<Erc20SupportT>::transfer(
            (*address).into(),
            (*caller).into(),
            (*value).try_into().map_err(|_| PrecompileFailure::Error {
                exit_status: ExitError::Other("value is out of bounds".into()),
            })?,
        )
        .map_err(process_dispatch_error::<Erc20SupportT>)?;

        let log = LogsBuilder::new(*address).log2(
            SELECTOR_LOG_DEPOSIT,
            *caller,
            EvmDataWriter::new().write(*value).build(),
        );
        record_log_cost(handle, &log)?;
        log.record(handle)?;

        Ok(succeed(EvmDataWriter::new().write(true).build()))
    }

    /// Simulate withdraw logic as IWETH-like contract.
    /// Do nothing.
    fn withdraw(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
        let mut input = handle.read_input()?;

        let caller = handle.context().caller;

        check_input(&mut input, 1)?;

        let amount: U256 = input.read()?;

        check_input_end(&mut input)?;

        let total_balance: U256 =
            pallet_erc20_support::Pallet::<Erc20SupportT>::balance_of(&caller.into()).into();

        if amount > total_balance {
            return Err(PrecompileFailure::Revert {
                exit_status: ExitRevert::Reverted,
                output: "trying to withdraw more than owned".into(),
            });
        }

        let log = LogsBuilder::new(handle.context().address).log2(
            SELECTOR_LOG_WITHDRAWAL,
            caller,
            EvmDataWriter::new().write(amount).build(),
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
/// The state-changing functions act on behalf of `handle.context().caller`, so reaching them via
/// `STATICCALL` would move the caller's funds from a context the EVM promises is read-only.
fn ensure_not_static(handle: &impl PrecompileHandle) -> EvmResult {
    if handle.is_static() {
        return Err(PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: b"cannot be called with STATICCALL".to_vec(),
        });
    }
    Ok(())
}

/// Rejects the call when value is sent along with it.
///
/// Only `deposit` makes use of the value; for the other functions the value would end up stuck
/// at the precompile address, as nothing can ever move it out from there.
fn ensure_no_value(handle: &impl PrecompileHandle) -> EvmResult {
    if !handle.context().apparent_value.is_zero() {
        return Err(PrecompileFailure::Revert {
            exit_status: ExitRevert::Reverted,
            output: b"function is not payable".to_vec(),
        });
    }
    Ok(())
}

/// A helper function to process dispatch related errors.
fn process_dispatch_error<Erc20SupportT: pallet_erc20_support::Config>(
    error: DispatchError,
) -> PrecompileFailure {
    if error == pallet_erc20_support::Error::SpendMoreThanAllowed::<Erc20SupportT>.into() {
        PrecompileFailure::Error {
            exit_status: ExitError::Other("spend more than allowed".into()),
        }
    } else {
        match error {
            DispatchError::Token(sp_runtime::TokenError::FundsUnavailable) => {
                PrecompileFailure::Error {
                    exit_status: ExitError::OutOfFund,
                }
            }
            _ => PrecompileFailure::Error {
                exit_status: ExitError::Other("unable to transfer funds".into()),
            },
        }
    }
}

/// A helper function to check expected arguments number.
fn check_input(input: &mut EvmDataReader, args_number: u8) -> Result<(), PrecompileFailure> {
    input
        .expect_arguments(args_number.into())
        .map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::Other("not expected arguments number".into()),
        })?;

    Ok(())
}

/// A helper function that verifies possible junk at the end of input.
fn check_input_end(input: &mut EvmDataReader) -> Result<(), PrecompileFailure> {
    let junk_data = input.read_till_end()?;
    if !junk_data.is_empty() {
        return Err(PrecompileFailure::Error {
            exit_status: ExitError::Other("junk at the end of input".into()),
        });
    }

    Ok(())
}
