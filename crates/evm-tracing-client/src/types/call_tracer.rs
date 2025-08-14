//! Call tracer explicitly types.

use codec::{Decode, Encode};
use serde::Serialize;
use sp_core::{H160, U256};

use super::{single::Call, CallResult};
use crate::serialization::*;

/// Call tracer call.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallTracerCall {
    /// From address.
    pub from: H160,
    /// Indices of parent calls. Used to build the Etherscan nested response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_address: Option<Vec<u32>>,
    /// Remaining gas in the runtime.
    pub gas: U256,
    /// Gas used by this context.
    pub gas_used: U256,
    /// Inner.
    #[serde(flatten)]
    pub inner: CallTracerInner,
    /// Calls.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<Call>,
}

/// Call tracer inner.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(untagged)]
pub enum CallTracerInner {
    /// Call.
    Call {
        /// Call type.
        #[serde(rename = "type", serialize_with = "opcode_serialize")]
        call_type: Vec<u8>,
        /// To.
        to: H160,
        /// Input.
        #[serde(serialize_with = "bytes_0x_serialize")]
        input: Vec<u8>,
        /// Call result.
        #[serde(flatten)]
        res: CallResult,
        /// Value.
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<U256>,
    },
    /// Create.
    Create {
        /// Call type.
        #[serde(rename = "type", serialize_with = "opcode_serialize")]
        call_type: Vec<u8>,
        /// Input.
        #[serde(serialize_with = "bytes_0x_serialize")]
        input: Vec<u8>,
        /// To.
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<H160>,
        #[serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "option_bytes_0x_serialize"
        )]
        /// Output.
        output: Option<Vec<u8>>,
        #[serde(
            skip_serializing_if = "Option::is_none",
            serialize_with = "option_string_serialize"
        )]
        /// Error.
        error: Option<Vec<u8>>,
        /// Value.
        value: U256,
    },
    /// Selfdestruct.
    SelfDestruct {
        /// Call type.
        #[serde(rename = "type", serialize_with = "opcode_serialize")]
        call_type: Vec<u8>,
        /// To.
        to: H160,
        /// Value.
        value: U256,
    },
}
