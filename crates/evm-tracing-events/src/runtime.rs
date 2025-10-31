//! EVM runtime events definitions.

extern crate alloc;

use codec::{Decode, Encode};
pub use evm::{ExitError, ExitReason, ExitSucceed};
use sp_core::{sp_std::vec::Vec, H160, H256, U256};

use crate::Context;
#[cfg(feature = "evm-tracing")]
use crate::StepEventFilter;

/// EVM stack.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Stack {
    /// Data.
    pub data: Vec<H256>,
    /// Limit.
    pub limit: u64,
}

impl From<&evm::Stack> for Stack {
    fn from(stack: &evm::Stack) -> Self {
        Self {
            data: stack.data().clone(),
            limit: stack.limit() as u64,
        }
    }
}

/// EVM memory.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Memory {
    /// Data.
    pub data: Vec<u8>,
    /// Effective length.
    pub effective_len: U256,
    /// Limit.
    pub limit: u64,
}

impl From<&evm::Memory> for Memory {
    fn from(memory: &evm::Memory) -> Self {
        Self {
            data: memory.data().clone(),
            effective_len: memory.effective_len(),
            limit: memory.limit() as u64,
        }
    }
}

/// Capture represents the result of execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Encode, Decode)]
pub enum Capture<E, T> {
    /// The machine has exited. It cannot be executed again.
    Exit(E),
    /// The machine has trapped. It is waiting for external information, and can
    /// be executed again.
    Trap(T),
}

/// A type alias representing trap data. Should hold the marshalled `Opcode`.
pub type Trap = evm::Opcode;

/// EVM runtime event.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum RuntimeEvent {
    /// Step.
    Step {
        /// Context.
        context: Context,
        /// Opcode.
        opcode: evm::Opcode,
        /// Position.
        position: Result<u64, ExitReason>,
        /// Stack.
        stack: Option<Stack>,
        /// Memory.
        memory: Option<Memory>,
    },
    /// Step result.
    StepResult {
        /// Result.
        result: Result<(), Capture<ExitReason, Trap>>,
        /// Return value.
        return_value: Vec<u8>,
    },
    /// Storage load.
    SLoad {
        /// Address.
        address: H160,
        /// Index.
        index: H256,
        /// Value.
        value: H256,
    },
    /// Storage store.
    SStore {
        /// Address.
        address: H160,
        /// Index.
        index: H256,
        /// Value.
        value: H256,
    },
}

#[cfg(feature = "evm-tracing")]
impl RuntimeEvent {
    /// Obtain `RuntimeEvent` from [`evm_runtime::tracing::Event`] based on provided
    /// step event filter.
    pub fn from_evm_event(event: evm_runtime::tracing::Event<'_>, filter: StepEventFilter) -> Self {
        match event {
            evm_runtime::tracing::Event::Step {
                context,
                opcode,
                position,
                stack,
                memory,
            } => Self::Step {
                context: context.clone().into(),
                opcode,
                position: match position {
                    Ok(position) => Ok(*position as u64),
                    Err(e) => Err(e.clone()),
                },
                stack: if filter.enable_stack {
                    Some(stack.into())
                } else {
                    None
                },
                memory: if filter.enable_memory {
                    Some(memory.into())
                } else {
                    None
                },
            },
            evm_runtime::tracing::Event::StepResult {
                result,
                return_value,
            } => Self::StepResult {
                result: match result {
                    Ok(_) => Ok(()),
                    Err(capture) => match capture {
                        evm::Capture::Exit(e) => Err(Capture::Exit(e.clone())),
                        evm::Capture::Trap(t) => Err(Capture::Trap(*t)),
                    },
                },
                return_value: return_value.to_vec(),
            },
            evm_runtime::tracing::Event::SLoad {
                address,
                index,
                value,
            } => Self::SLoad {
                address,
                index,
                value,
            },
            evm_runtime::tracing::Event::SStore {
                address,
                index,
                value,
            } => Self::SStore {
                address,
                index,
                value,
            },
        }
    }
}
