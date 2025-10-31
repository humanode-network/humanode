//! Single transaction related types.

use codec::{Decode, Encode};
use serde::Serialize;
use sp_core::{sp_std::collections::btree_map::BTreeMap, H256, U256};

use crate::serialization::*;

/// Call.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Call {
    /// Blockscout call.
    Blockscout(super::blockscout::BlockscoutCall),
    /// Call tracer.
    CallTracer(super::call_tracer::CallTracerCall),
}

/// Trace type.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode)]
pub enum TraceType {
    /// Classic geth with no javascript based tracing.
    Raw {
        /// Disable storage flag.
        disable_storage: bool,
        /// Disable memory flag.
        disable_memory: bool,
        /// Disable stack flag.
        disable_stack: bool,
    },
    /// List of calls and subcalls formatted with an input tracer (i.e. callTracer or Blockscout).
    CallList,
    /// A single block trace. Use in `debug_traceTransactionByNumber` / `traceTransactionByHash`.
    Block,
}

/// Single transaction trace.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum TransactionTrace {
    /// Classical output of `debug_trace`.
    #[serde(rename_all = "camelCase")]
    Raw {
        /// Gas.
        gas: U256,
        /// Return value.
        #[serde(with = "hex")]
        return_value: Vec<u8>,
        /// Logs.
        struct_logs: Vec<RawStepLog>,
    },
    /// Matches the formatter used by Blockscout.
    CallList(Vec<Call>),
    /// Used by Geth's callTracer.
    CallListNested(Call),
}

/// Raw step log.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawStepLog {
    /// Depth.
    #[serde(serialize_with = "u256_serialize")]
    pub depth: U256,
    /// Gas.
    #[serde(serialize_with = "u256_serialize")]
    pub gas: U256,
    /// Gas cost.
    #[serde(serialize_with = "u256_serialize")]
    pub gas_cost: U256,
    /// Memory.
    #[serde(
        serialize_with = "seq_h256_serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub memory: Option<Vec<H256>>,
    /// Op.
    #[serde(serialize_with = "opcode_serialize")]
    pub op: evm::Opcode,
    /// Pc.
    #[serde(serialize_with = "u256_serialize")]
    pub pc: U256,
    /// Stack.
    #[serde(
        serialize_with = "seq_h256_serialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub stack: Option<Vec<H256>>,
    /// Storage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<BTreeMap<H256, H256>>,
}
