//! Marshalled opcode definition and implementations.

use codec::{Decode, Encode};
use smallvec::{smallvec, SmallVec};
use sp_core::sp_std::{vec, vec::Vec};

/// Marshalled opcode.
///
/// 8 cause the longest is 13 and the epmeric estimate of max length
/// for the most popular ones in 7-ish.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MarshalledOpcode(pub SmallVec<[u8; 8]>);

impl From<evm::Opcode> for MarshalledOpcode {
    fn from(opcode: evm::Opcode) -> Self {
        MarshalledOpcode(smallvec![opcode.as_u8()])
    }
}

impl MarshalledOpcode {
    /// Returns `CREATE` marshalled opcode.
    pub fn create_opcode() -> Self {
        evm::Opcode::CREATE.into()
    }

    /// Returns `CALL` marshalled opcode.
    pub fn call_opcode() -> Self {
        evm::Opcode::CALL.into()
    }

    /// Returns `CALLCODE` marshalled opcode.
    pub fn callcode_opcode() -> Self {
        evm::Opcode::CALLCODE.into()
    }

    /// Returns `DELEGATECALL` marshalled opcode.
    pub fn delegatecall_opcode() -> Self {
        evm::Opcode::DELEGATECALL.into()
    }

    /// Returns `STATICCALL` marshalled opcode.
    pub fn staticcall_opcode() -> Self {
        evm::Opcode::STATICCALL.into()
    }

    /// Returns `SELFDESTRUCT` marshalled opcode.
    pub fn selfdestruct_opcode() -> Self {
        evm::Opcode::SUICIDE.into()
    }

    /// Check whether it's a known opcode or not. In case it's a known one,
    /// return the name of the opcode then.
    pub fn known_name(&self) -> Option<&'static str> {
        if self.0.len() != 1 {
            return None;
        }

        let opcode = self.0[0];

        Some(match opcode {
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
}

impl core::fmt::Display for MarshalledOpcode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.known_name() {
            Some(s) => write!(f, "{}", s.to_uppercase()),
            None => {
                write!(f, "UNKNOWN(0x")?;

                for byte in self.0.as_slice() {
                    write!(f, "{:02x}", byte)?;
                }

                write!(f, ")")
            }
        }
    }
}

impl Encode for MarshalledOpcode {
    fn encode(&self) -> Vec<u8> {
        self.0.clone().to_vec()
    }
}

impl Decode for MarshalledOpcode {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let buffer_len = input
            .remaining_len()?
            .ok_or("Encoded opcode shouldn't be empty")?;

        let mut buffer = vec![0_u8; buffer_len];
        input.read(&mut buffer)?;

        Ok(MarshalledOpcode(SmallVec::from_slice(&buffer)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_works() {
        let cases = [
            (MarshalledOpcode::create_opcode(), vec![0xf0]),
            (MarshalledOpcode::selfdestruct_opcode(), vec![0xff]),
            (
                MarshalledOpcode(smallvec![0x11, 0x22, 0x33]),
                vec![0x11, 0x22, 0x33],
            ),
        ];

        for (opcode, expected_encoded) in cases {
            let encoded = opcode.encode();
            assert_eq!(encoded, expected_encoded);
            assert_eq!(MarshalledOpcode::decode(&mut &encoded[..]).unwrap(), opcode);
        }
    }

    #[test]
    fn display_works() {
        assert_eq!(MarshalledOpcode::create_opcode().to_string(), "CREATE");

        assert_eq!(
            MarshalledOpcode::selfdestruct_opcode().to_string(),
            "SELFDESTRUCT"
        );

        assert_eq!(
            MarshalledOpcode(smallvec![0x11, 0x22, 0x33]).to_string(),
            "UNKNOWN(0x112233)"
        );
    }
}
