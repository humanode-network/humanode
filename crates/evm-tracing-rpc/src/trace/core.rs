//! Core.

use evm_tracing_client::types::block::TransactionTrace;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde::Deserialize;
use sp_core::H160;

use crate::types::RequestBlockId;

/// Filter request.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterRequest {
    /// From this block.
    pub from_block: Option<RequestBlockId>,
    /// To this block.
    pub to_block: Option<RequestBlockId>,
    /// Sent from these addresses.
    pub from_address: Option<Vec<H160>>,
    /// Sent to these addresses.
    pub to_address: Option<Vec<H160>>,
    /// The offset trace number.
    pub after: Option<u32>,
    /// Integer number of traces to display in a batch.
    pub count: Option<u32>,
}

#[rpc(server)]
pub trait Trace {
    /// Filter.
    #[method(name = "trace_filter")]
    async fn filter(&self, filter: FilterRequest) -> RpcResult<Vec<TransactionTrace>>;
}
