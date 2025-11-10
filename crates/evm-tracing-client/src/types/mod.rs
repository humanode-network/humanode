//! EVM tracing types.

extern crate alloc;

use codec::{Decode, Encode};
use evm_tracing_events::MarshalledOpcode;
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
    pub fn from(opcode: MarshalledOpcode) -> Option<Self> {
        match &opcode.to_string()[..] {
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
    let chunk_size = 32;

    memory
        .chunks(chunk_size)
        .map(|chunk| {
            let mut buffer = [0u8; 32];
            buffer[chunk_size
                .checked_sub(chunk.len())
                .expect("valid operation; qed")..]
                .copy_from_slice(chunk);
            H256::from_slice(&buffer)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{convert_memory, H256};

    #[test]
    fn convert_memory_empty_input() {
        let input = vec![];
        let output = convert_memory(input);
        assert!(output.is_empty());
    }

    #[test]
    fn convert_memory_muptiply_of_32_bytes() {
        let input = vec![1u8; 64];
        let output = convert_memory(input);

        assert_eq!(output.len(), 2);
        assert_eq!(output[0], H256::from_slice(&[1u8; 32]));
        assert_eq!(output[1], H256::from_slice(&[1u8; 32]));
    }

    #[test]
    fn convert_memory_less_than_32_bytes() {
        let input = vec![2u8; 10];
        let output = convert_memory(input);

        let mut expected_partial = [0u8; 32];
        expected_partial[22..].copy_from_slice(&[2u8; 10]);

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], H256::from_slice(&expected_partial));
    }

    #[test]
    fn convert_memory_more_than_32_bytes() {
        let input = vec![3u8; 42];
        let output = convert_memory(input);

        let mut expected_partial = [0u8; 32];
        expected_partial[22..].copy_from_slice(&[3u8; 10]);

        assert_eq!(output.len(), 2);
        assert_eq!(output[0], H256::from_slice(&[3u8; 32]));
        assert_eq!(output[1], H256::from_slice(&expected_partial));
    }
}
