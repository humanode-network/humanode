//! Block transaction related types.

use codec::{Decode, Encode};
use serde::Serialize;
use sp_core::{H160, H256, U256};

use super::{CallType, CreateType};
use crate::serialization::*;

/// Block transaction trace.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransactionTrace {
    /// Tx hash.
    #[serde(serialize_with = "h256_0x_serialize")]
    pub tx_hash: H256,
    /// Result.
    pub result: super::single::TransactionTrace,
    /// Tx position.
    #[serde(skip_serializing)]
    pub tx_position: u32,
}

/// Transaction trace.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTrace {
    /// Transaction trace action.
    #[serde(flatten)]
    pub action: TransactionTraceAction,
    /// Block hash.
    #[serde(serialize_with = "h256_0x_serialize")]
    pub block_hash: H256,
    /// Block number.
    pub block_number: u32,
    /// Output.
    #[serde(flatten)]
    pub output: TransactionTraceOutput,
    /// Subtraces.
    pub subtraces: u32,
    /// Trace address.
    pub trace_address: Vec<u32>,
    /// Transaction hash.
    #[serde(serialize_with = "h256_0x_serialize")]
    pub transaction_hash: H256,
    /// Transaction position.
    pub transaction_position: u32,
}

/// Transaction trace action.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "action")]
pub enum TransactionTraceAction {
    /// Call.
    #[serde(rename_all = "camelCase")]
    Call {
        /// Call type.
        call_type: CallType,
        /// From.
        from: H160,
        /// Gas.
        gas: U256,
        /// Input.
        #[serde(serialize_with = "bytes_0x_serialize")]
        input: Vec<u8>,
        /// To.
        to: H160,
        /// Value.
        value: U256,
    },
    /// Create.
    #[serde(rename_all = "camelCase")]
    Create {
        /// Creation method.
        creation_method: CreateType,
        /// From.
        from: H160,
        /// Gas.
        gas: U256,
        /// Init.
        #[serde(serialize_with = "bytes_0x_serialize")]
        init: Vec<u8>,
        /// Value.
        value: U256,
    },
    /// Suicide.
    #[serde(rename_all = "camelCase")]
    Suicide {
        /// Address.
        address: H160,
        /// Balance.
        balance: U256,
        /// Refund address.
        refund_address: H160,
    },
}

/// Transaction trace output.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionTraceOutput {
    /// Result.
    Result(TransactionTraceResult),
    /// Error.
    Error(#[serde(serialize_with = "string_serialize")] Vec<u8>),
}

/// Transaction trace result.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum TransactionTraceResult {
    /// Call.
    #[serde(rename_all = "camelCase")]
    Call {
        /// Gas used.
        gas_used: U256,
        /// Output.
        #[serde(serialize_with = "bytes_0x_serialize")]
        output: Vec<u8>,
    },
    /// Create.
    #[serde(rename_all = "camelCase")]
    Create {
        /// Address.
        address: H160,
        /// Code.
        #[serde(serialize_with = "bytes_0x_serialize")]
        code: Vec<u8>,
        /// Gas used.
        gas_used: U256,
    },
    /// Suicide.
    Suicide,
}
