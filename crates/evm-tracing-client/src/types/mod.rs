//! EVM tracing types.

extern crate alloc;

use codec::{Decode, Encode};
use serde::Serialize;
use sp_core::{H160, H256};

pub mod block;
pub mod blockscout;
pub mod call_tracer;
pub mod single;

use crate::serialization::*;

/// Call result.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CallResult {
    /// Output.
    Output(#[serde(serialize_with = "bytes_0x_serialize")] Vec<u8>),
    /// Error.
    Error(#[serde(serialize_with = "string_serialize")] Vec<u8>),
}

/// Create result.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum CreateResult {
    /// Error.
    Error {
        /// Error bytes.
        #[serde(serialize_with = "string_serialize")]
        error: Vec<u8>,
    },
    /// Success.
    Success {
        /// Created contract hash value,
        #[serde(rename = "createdContractAddressHash")]
        created_contract_address_hash: H160,
        /// Created contract code.
        #[serde(serialize_with = "bytes_0x_serialize", rename = "createdContractCode")]
        created_contract_code: Vec<u8>,
    },
}

/// Call type.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CallType {
    /// Call.
    Call,
    /// Call code.
    CallCode,
    /// Delegate call.
    DelegateCall,
    /// Static call.
    StaticCall,
}

/// Create type.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CreateType {
    /// Create type.
    Create,
}

/// Context type.
#[derive(Debug)]
pub enum ContextType {
    /// Call type.
    Call(CallType),
    /// Create type.
    Create,
}

impl ContextType {
    /// Obtain context type from opcode.
    pub fn from(opcode: Vec<u8>) -> Option<Self> {
        let opcode = match alloc::str::from_utf8(&opcode[..]) {
            Ok(op) => op.to_uppercase(),
            _ => return None,
        };
        match &opcode[..] {
            "CREATE" | "CREATE2" => Some(ContextType::Create),
            "CALL" => Some(ContextType::Call(CallType::Call)),
            "CALLCODE" => Some(ContextType::Call(CallType::CallCode)),
            "DELEGATECALL" => Some(ContextType::Call(CallType::DelegateCall)),
            "STATICCALL" => Some(ContextType::Call(CallType::StaticCall)),
            _ => None,
        }
    }
}

/// Memory converter.
pub fn convert_memory(memory: Vec<u8>) -> Vec<H256> {
    let size = 32;
    memory
        .chunks(size)
        .map(|c| {
            let mut msg = [0u8; 32];
            let chunk = c.len();
            if chunk < size {
                let left = size.checked_sub(chunk).expect("valid op; qed.");
                let remainder = vec![0; left];
                msg[0..left].copy_from_slice(&remainder[..]);
                msg[left..size].copy_from_slice(c);
            } else {
                msg[0..size].copy_from_slice(c)
            }
            H256::from_slice(&msg[..])
        })
        .collect()
}
