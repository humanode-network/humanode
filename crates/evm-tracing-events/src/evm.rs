//! EVM explicitly events definitions.

use codec::{Decode, Encode};
use evm::ExitReason;
use sp_core::{sp_std::vec::Vec, H160, H256, U256};

use crate::Context;

/// EVM transfer.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Transfer {
    /// Source address.
    pub source: H160,
    /// Target address.
    pub target: H160,
    /// Transfer value.
    pub value: U256,
}

impl From<evm_runtime::Transfer> for Transfer {
    fn from(transfer: evm_runtime::Transfer) -> Self {
        Self {
            source: transfer.source,
            target: transfer.target,
            value: transfer.value,
        }
    }
}

/// EVM create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`.
    Legacy {
        /// Caller of the create.
        caller: H160,
    },
    /// Create scheme of `CREATE2`.
    Create2 {
        /// Caller of the create.
        caller: H160,
        /// Code hash.
        code_hash: H256,
        /// Salt.
        salt: H256,
    },
    /// Create at a fixed location.
    Fixed(H160),
}

impl From<evm_runtime::CreateScheme> for CreateScheme {
    fn from(create_scheme: evm_runtime::CreateScheme) -> Self {
        match create_scheme {
            evm_runtime::CreateScheme::Legacy { caller } => Self::Legacy { caller },
            evm_runtime::CreateScheme::Create2 {
                caller,
                code_hash,
                salt,
            } => Self::Create2 {
                caller,
                code_hash,
                salt,
            },
            evm_runtime::CreateScheme::Fixed(address) => Self::Fixed(address),
        }
    }
}

/// EVM event.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum EvmEvent {
    /// Call.
    Call {
        /// Code address.
        code_address: H160,
        /// Transfer.
        transfer: Option<Transfer>,
        /// Input.
        input: Vec<u8>,
        /// Target gas.
        target_gas: Option<u64>,
        /// Is static flag.
        is_static: bool,
        /// Context.
        context: Context,
    },
    /// Create.
    Create {
        /// Caller.
        caller: H160,
        /// Address.
        address: H160,
        /// Scheme.
        scheme: CreateScheme,
        /// Value.
        value: U256,
        /// Init code.
        init_code: Vec<u8>,
        /// Target gas.
        target_gas: Option<u64>,
    },
    /// Suicide.
    Suicide {
        /// Address.
        address: H160,
        /// Target.
        target: H160,
        /// Balance.
        balance: U256,
    },
    /// Exit.
    Exit {
        /// Reason.
        reason: ExitReason,
        /// Return value.
        return_value: Vec<u8>,
    },
    /// Transact call.
    TransactCall {
        /// Caller.
        caller: H160,
        /// Address.
        address: H160,
        /// Value.
        value: U256,
        /// Data.
        data: Vec<u8>,
        /// Gas limit.
        gas_limit: u64,
    },
    /// Transact create.
    TransactCreate {
        /// Caller.
        caller: H160,
        /// Value.
        value: U256,
        /// Init code.
        init_code: Vec<u8>,
        /// Gas limit.
        gas_limit: u64,
        /// Address.
        address: H160,
    },
    /// Transact create2.
    TransactCreate2 {
        /// Caller.
        caller: H160,
        /// Value.
        value: U256,
        /// Init code.
        init_code: Vec<u8>,
        /// Salt.
        salt: H256,
        /// Gas limit.
        gas_limit: u64,
        /// Address.
        address: H160,
    },
    /// Precompile subcall.
    PrecompileSubcall {
        /// Code address.
        code_address: H160,
        /// Transfer.
        transfer: Option<Transfer>,
        /// Input.
        input: Vec<u8>,
        /// Target.
        target_gas: Option<u64>,
        /// Is static flag.
        is_static: bool,
        /// Context.
        context: Context,
    },
}

#[cfg(feature = "evm-tracing")]
impl<'a> From<evm::tracing::Event<'a>> for EvmEvent {
    fn from(event: evm::tracing::Event<'a>) -> Self {
        match event {
            evm::tracing::Event::Call {
                code_address,
                transfer,
                input,
                target_gas,
                is_static,
                context,
            } => Self::Call {
                code_address,
                transfer: transfer.as_ref().map(|transfer| transfer.clone().into()),
                input: input.to_vec(),
                target_gas,
                is_static,
                context: context.clone().into(),
            },
            evm::tracing::Event::Create {
                caller,
                address,
                scheme,
                value,
                init_code,
                target_gas,
            } => Self::Create {
                caller,
                address,
                scheme: scheme.into(),
                value,
                init_code: init_code.to_vec(),
                target_gas,
            },
            evm::tracing::Event::Suicide {
                address,
                target,
                balance,
            } => Self::Suicide {
                address,
                target,
                balance,
            },
            evm::tracing::Event::Exit {
                reason,
                return_value,
            } => Self::Exit {
                reason: reason.clone(),
                return_value: return_value.to_vec(),
            },
            evm::tracing::Event::TransactCall {
                caller,
                address,
                value,
                data,
                gas_limit,
            } => Self::TransactCall {
                caller,
                address,
                value,
                data: data.to_vec(),
                gas_limit,
            },
            evm::tracing::Event::TransactCreate {
                caller,
                value,
                init_code,
                gas_limit,
                address,
            } => Self::TransactCreate {
                caller,
                value,
                init_code: init_code.to_vec(),
                gas_limit,
                address,
            },
            evm::tracing::Event::TransactCreate2 {
                caller,
                value,
                init_code,
                salt,
                gas_limit,
                address,
            } => Self::TransactCreate2 {
                caller,
                value,
                init_code: init_code.to_vec(),
                salt,
                gas_limit,
                address,
            },
            evm::tracing::Event::PrecompileSubcall {
                code_address,
                transfer,
                input,
                target_gas,
                is_static,
                context,
            } => Self::PrecompileSubcall {
                code_address,
                transfer: transfer.as_ref().map(|transfer| transfer.clone().into()),
                input: input.to_vec(),
                target_gas,
                is_static,
                context: context.clone().into(),
            },
        }
    }
}
