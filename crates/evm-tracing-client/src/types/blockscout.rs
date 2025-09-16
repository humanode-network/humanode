//! Blockscout explicitly types.

use codec::{Decode, Encode};
use serde::Serialize;
use sp_core::{H160, U256};

use super::{CallResult, CallType, CreateResult};
use crate::serialization::*;

/// Blockcout call.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockscoutCall {
    /// From address.
    pub from: H160,
    /// Indices of parent calls.
    pub trace_address: Vec<u32>,
    /// Number of children calls.
    /// Not needed for Blockscout, but needed for `crate::block`
    /// types that are build from this type.
    #[serde(skip)]
    pub subtraces: u32,
    /// Sends funds to the (payable) function.
    pub value: U256,
    /// Remaining gas in the runtime.
    pub gas: U256,
    /// Gas used by this context.
    pub gas_used: U256,
    /// Inner.
    #[serde(flatten)]
    pub inner: BlockscoutCallInner,
}

/// Blockscout call inner.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum BlockscoutCallInner {
    /// Call.
    Call {
        /// Type of call.
        #[serde(rename(serialize = "callType"))]
        call_type: CallType,
        /// To.
        to: H160,
        /// Input.
        #[serde(serialize_with = "bytes_0x_serialize")]
        input: Vec<u8>,
        /// Call result.
        #[serde(flatten)]
        res: CallResult,
    },
    /// Create.
    Create {
        /// Init.
        #[serde(serialize_with = "bytes_0x_serialize")]
        init: Vec<u8>,
        /// Create result.
        #[serde(flatten)]
        res: CreateResult,
    },
    /// Selfdestruct.
    SelfDestruct {
        /// Balance.
        #[serde(skip)]
        balance: U256,
        /// To.
        to: H160,
    },
}
