//! Core.

use ethereum::AccessListItem;
use evm_tracing_client::types::{block, single};
use fc_rpc_core::types::Bytes;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde::Deserialize;
use sp_core::{H160, H256, U256};

use crate::types::RequestBlockId;

/// Trace params.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceParams {
    /// Disable storage flag.
    pub disable_storage: Option<bool>,
    /// Disable memory flag.
    pub disable_memory: Option<bool>,
    /// Disable stack flag.
    pub disable_stack: Option<bool>,
    /// Javascript tracer (we just check if it's Blockscout tracer string).
    pub tracer: Option<String>,
    /// Timeout.
    pub timeout: Option<String>,
}

/// Trace call params.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallParams {
    /// Sender.
    pub from: Option<H160>,
    /// Recipient.
    pub to: H160,
    /// Gas price.
    pub gas_price: Option<U256>,
    /// Max `BaseFeePerGas` the user is willing to pay.
    pub max_fee_per_gas: Option<U256>,
    /// The miner's tip.
    pub max_priority_fee_per_gas: Option<U256>,
    /// Gas.
    pub gas: Option<U256>,
    /// Value of transaction in wei.
    pub value: Option<U256>,
    /// Additional data sent with transaction.
    pub data: Option<Bytes>,
    /// Nonce.
    pub nonce: Option<U256>,
    /// EIP-2930 access list.
    pub access_list: Option<Vec<AccessListItem>>,
    /// EIP-2718 type.
    #[serde(rename = "type")]
    pub transaction_type: Option<U256>,
}

#[rpc(server)]
pub trait Debug {
    /// Trace transaction.
    #[method(name = "debug_traceTransaction")]
    async fn trace_transaction(
        &self,
        transaction_hash: H256,
        params: Option<TraceParams>,
    ) -> RpcResult<single::TransactionTrace>;

    /// Trace call.
    #[method(name = "debug_traceCall")]
    async fn trace_call(
        &self,
        call_params: TraceCallParams,
        id: RequestBlockId,
        params: Option<TraceParams>,
    ) -> RpcResult<single::TransactionTrace>;

    /// Trace block.
    #[method(name = "debug_traceBlockByNumber", aliases = ["debug_traceBlockByHash"])]
    async fn trace_block(
        &self,
        id: RequestBlockId,
        params: Option<TraceParams>,
    ) -> RpcResult<Vec<block::BlockTransactionTrace>>;
}
