//! EVM runtime events definitions.

extern crate alloc;

use codec::{Decode, Encode};
use evm::ExitReason;
#[cfg(feature = "evm-tracing")]
use evm::Opcode;
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
    let tmp;
    let out = match opcode {
        Opcode(0) => "Stop",
        Opcode(1) => "Add",
        Opcode(2) => "Mul",
        Opcode(3) => "Sub",
        Opcode(4) => "Div",
        Opcode(5) => "SDiv",
        Opcode(6) => "Mod",
        Opcode(7) => "SMod",
        Opcode(8) => "AddMod",
        Opcode(9) => "MulMod",
        Opcode(10) => "Exp",
        Opcode(11) => "SignExtend",
        Opcode(16) => "Lt",
        Opcode(17) => "Gt",
        Opcode(18) => "Slt",
        Opcode(19) => "Sgt",
        Opcode(20) => "Eq",
        Opcode(21) => "IsZero",
        Opcode(22) => "And",
        Opcode(23) => "Or",
        Opcode(24) => "Xor",
        Opcode(25) => "Not",
        Opcode(26) => "Byte",
        Opcode(27) => "Shl",
        Opcode(28) => "Shr",
        Opcode(29) => "Sar",
        Opcode(32) => "Keccak256",
        Opcode(48) => "Address",
        Opcode(49) => "Balance",
        Opcode(50) => "Origin",
        Opcode(51) => "Caller",
        Opcode(52) => "CallValue",
        Opcode(53) => "CallDataLoad",
        Opcode(54) => "CallDataSize",
        Opcode(55) => "CallDataCopy",
        Opcode(56) => "CodeSize",
        Opcode(57) => "CodeCopy",
        Opcode(58) => "GasPrice",
        Opcode(59) => "ExtCodeSize",
        Opcode(60) => "ExtCodeCopy",
        Opcode(61) => "ReturnDataSize",
        Opcode(62) => "ReturnDataCopy",
        Opcode(63) => "ExtCodeHash",
        Opcode(64) => "BlockHash",
        Opcode(65) => "Coinbase",
        Opcode(66) => "Timestamp",
        Opcode(67) => "Number",
        Opcode(68) => "Difficulty",
        Opcode(69) => "GasLimit",
        Opcode(70) => "ChainId",
        Opcode(80) => "Pop",
        Opcode(81) => "MLoad",
        Opcode(82) => "MStore",
        Opcode(83) => "MStore8",
        Opcode(84) => "SLoad",
        Opcode(85) => "SStore",
        Opcode(86) => "Jump",
        Opcode(87) => "JumpI",
        Opcode(88) => "GetPc",
        Opcode(89) => "MSize",
        Opcode(90) => "Gas",
        Opcode(91) => "JumpDest",
        Opcode(92) => "TLoad",
        Opcode(93) => "TStore",
        Opcode(94) => "MCopy",
        Opcode(96) => "Push1",
        Opcode(97) => "Push2",
        Opcode(98) => "Push3",
        Opcode(99) => "Push4",
        Opcode(100) => "Push5",
        Opcode(101) => "Push6",
        Opcode(102) => "Push7",
        Opcode(103) => "Push8",
        Opcode(104) => "Push9",
        Opcode(105) => "Push10",
        Opcode(106) => "Push11",
        Opcode(107) => "Push12",
        Opcode(108) => "Push13",
        Opcode(109) => "Push14",
        Opcode(110) => "Push15",
        Opcode(111) => "Push16",
        Opcode(112) => "Push17",
        Opcode(113) => "Push18",
        Opcode(114) => "Push19",
        Opcode(115) => "Push20",
        Opcode(116) => "Push21",
        Opcode(117) => "Push22",
        Opcode(118) => "Push23",
        Opcode(119) => "Push24",
        Opcode(120) => "Push25",
        Opcode(121) => "Push26",
        Opcode(122) => "Push27",
        Opcode(123) => "Push28",
        Opcode(124) => "Push29",
        Opcode(125) => "Push30",
        Opcode(126) => "Push31",
        Opcode(127) => "Push32",
        Opcode(128) => "Dup1",
        Opcode(129) => "Dup2",
        Opcode(130) => "Dup3",
        Opcode(131) => "Dup4",
        Opcode(132) => "Dup5",
        Opcode(133) => "Dup6",
        Opcode(134) => "Dup7",
        Opcode(135) => "Dup8",
        Opcode(136) => "Dup9",
        Opcode(137) => "Dup10",
        Opcode(138) => "Dup11",
        Opcode(139) => "Dup12",
        Opcode(140) => "Dup13",
        Opcode(141) => "Dup14",
        Opcode(142) => "Dup15",
        Opcode(143) => "Dup16",
        Opcode(144) => "Swap1",
        Opcode(145) => "Swap2",
        Opcode(146) => "Swap3",
        Opcode(147) => "Swap4",
        Opcode(148) => "Swap5",
        Opcode(149) => "Swap6",
        Opcode(150) => "Swap7",
        Opcode(151) => "Swap8",
        Opcode(152) => "Swap9",
        Opcode(153) => "Swap10",
        Opcode(154) => "Swap11",
        Opcode(155) => "Swap12",
        Opcode(156) => "Swap13",
        Opcode(157) => "Swap14",
        Opcode(158) => "Swap15",
        Opcode(159) => "Swap16",
        Opcode(160) => "Log0",
        Opcode(161) => "Log1",
        Opcode(162) => "Log2",
        Opcode(163) => "Log3",
        Opcode(164) => "Log4",
        Opcode(176) => "JumpTo",
        Opcode(177) => "JumpIf",
        Opcode(178) => "JumpSub",
        Opcode(180) => "JumpSubv",
        Opcode(181) => "BeginSub",
        Opcode(182) => "BeginData",
        Opcode(184) => "ReturnSub",
        Opcode(185) => "PutLocal",
        Opcode(186) => "GetLocal",
        Opcode(225) => "SLoadBytes",
        Opcode(226) => "SStoreBytes",
        Opcode(227) => "SSize",
        Opcode(240) => "Create",
        Opcode(241) => "Call",
        Opcode(242) => "CallCode",
        Opcode(243) => "Return",
        Opcode(244) => "DelegateCall",
        Opcode(245) => "Create2",
        Opcode(250) => "StaticCall",
        Opcode(252) => "TxExecGas",
        Opcode(253) => "Revert",
        Opcode(254) => "Invalid",
        Opcode(255) => "SelfDestruct",
        Opcode(n) => {
            tmp = alloc::format!("Unknown({})", n);
            &tmp
        }
    };
    out.as_bytes().to_vec()
}
