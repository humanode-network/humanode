//! EVM runtime events definitions.

extern crate alloc;

use codec::{Decode, Encode};
#[cfg(feature = "evm-tracing")]
use evm::Opcode;
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
pub type Trap = Vec<u8>;

/// EVM runtime event.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub enum RuntimeEvent {
    /// Step.
    Step {
        /// Context.
        context: Context,
        /// Opcode.
        opcode: Vec<u8>,
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
                opcode: opcodes_string(opcode),
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
                        evm::Capture::Trap(t) => Err(Capture::Trap(opcodes_string(*t))),
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

/// Converts an `Opcode` into its name, stored in a `Vec<u8>`.
#[cfg(feature = "evm-tracing")]
pub fn opcodes_string(opcode: Opcode) -> Vec<u8> {
    match opcode_str(opcode) {
        Some(s) => s.into(),
        None => alloc::format!("Unknown({})", opcode.0).into(),
    }
}

/// Converts an `Opcode` into its name.
#[cfg(feature = "evm-tracing")]
pub fn opcode_str(opcode: Opcode) -> Option<&'static str> {
    Some(match opcode.0 {
        0 => "Stop",
        1 => "Add",
        2 => "Mul",
        3 => "Sub",
        4 => "Div",
        5 => "SDiv",
        6 => "Mod",
        7 => "SMod",
        8 => "AddMod",
        9 => "MulMod",
        10 => "Exp",
        11 => "SignExtend",
        16 => "Lt",
        17 => "Gt",
        18 => "Slt",
        19 => "Sgt",
        20 => "Eq",
        21 => "IsZero",
        22 => "And",
        23 => "Or",
        24 => "Xor",
        25 => "Not",
        26 => "Byte",
        27 => "Shl",
        28 => "Shr",
        29 => "Sar",
        32 => "Keccak256",
        48 => "Address",
        49 => "Balance",
        50 => "Origin",
        51 => "Caller",
        52 => "CallValue",
        53 => "CallDataLoad",
        54 => "CallDataSize",
        55 => "CallDataCopy",
        56 => "CodeSize",
        57 => "CodeCopy",
        58 => "GasPrice",
        59 => "ExtCodeSize",
        60 => "ExtCodeCopy",
        61 => "ReturnDataSize",
        62 => "ReturnDataCopy",
        63 => "ExtCodeHash",
        64 => "BlockHash",
        65 => "Coinbase",
        66 => "Timestamp",
        67 => "Number",
        68 => "Difficulty",
        69 => "GasLimit",
        70 => "ChainId",
        80 => "Pop",
        81 => "MLoad",
        82 => "MStore",
        83 => "MStore8",
        84 => "SLoad",
        85 => "SStore",
        86 => "Jump",
        87 => "JumpI",
        88 => "GetPc",
        89 => "MSize",
        90 => "Gas",
        91 => "JumpDest",
        92 => "TLoad",
        93 => "TStore",
        94 => "MCopy",
        96 => "Push1",
        97 => "Push2",
        98 => "Push3",
        99 => "Push4",
        100 => "Push5",
        101 => "Push6",
        102 => "Push7",
        103 => "Push8",
        104 => "Push9",
        105 => "Push10",
        106 => "Push11",
        107 => "Push12",
        108 => "Push13",
        109 => "Push14",
        110 => "Push15",
        111 => "Push16",
        112 => "Push17",
        113 => "Push18",
        114 => "Push19",
        115 => "Push20",
        116 => "Push21",
        117 => "Push22",
        118 => "Push23",
        119 => "Push24",
        120 => "Push25",
        121 => "Push26",
        122 => "Push27",
        123 => "Push28",
        124 => "Push29",
        125 => "Push30",
        126 => "Push31",
        127 => "Push32",
        128 => "Dup1",
        129 => "Dup2",
        130 => "Dup3",
        131 => "Dup4",
        132 => "Dup5",
        133 => "Dup6",
        134 => "Dup7",
        135 => "Dup8",
        136 => "Dup9",
        137 => "Dup10",
        138 => "Dup11",
        139 => "Dup12",
        140 => "Dup13",
        141 => "Dup14",
        142 => "Dup15",
        143 => "Dup16",
        144 => "Swap1",
        145 => "Swap2",
        146 => "Swap3",
        147 => "Swap4",
        148 => "Swap5",
        149 => "Swap6",
        150 => "Swap7",
        151 => "Swap8",
        152 => "Swap9",
        153 => "Swap10",
        154 => "Swap11",
        155 => "Swap12",
        156 => "Swap13",
        157 => "Swap14",
        158 => "Swap15",
        159 => "Swap16",
        160 => "Log0",
        161 => "Log1",
        162 => "Log2",
        163 => "Log3",
        164 => "Log4",
        176 => "JumpTo",
        177 => "JumpIf",
        178 => "JumpSub",
        180 => "JumpSubv",
        181 => "BeginSub",
        182 => "BeginData",
        184 => "ReturnSub",
        185 => "PutLocal",
        186 => "GetLocal",
        225 => "SLoadBytes",
        226 => "SStoreBytes",
        227 => "SSize",
        240 => "Create",
        241 => "Call",
        242 => "CallCode",
        243 => "Return",
        244 => "DelegateCall",
        245 => "Create2",
        250 => "StaticCall",
        252 => "TxExecGas",
        253 => "Revert",
        254 => "Invalid",
        255 => "SelfDestruct",
        _ => return None,
    })
}
