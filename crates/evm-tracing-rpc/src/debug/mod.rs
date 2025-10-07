//! Debug related implementation.

use core::{DebugServer, TraceCallParams, TraceParams};

use evm_tracing_client::types::{
    block::{self, BlockTransactionTrace},
    single,
};
use fc_rpc::internal_err;
use jsonrpsee::core::{async_trait, RpcResult};
use sc_utils::mpsc::TracingUnboundedSender;
use sp_core::H256;
use tokio::sync::oneshot;

pub mod core;
mod handler;

pub use handler::DebugHandler;

use crate::types::RequestBlockId;

/// Requester input.
#[allow(clippy::large_enum_variant)]
pub enum RequesterInput {
    /// Call.
    Call((RequestBlockId, TraceCallParams)),
    /// Transaction.
    Transaction(H256),
    /// Block.
    Block(RequestBlockId),
}

/// Response.
pub enum Response {
    /// Single transaction data.
    Single(single::TransactionTrace),
    /// Block data.
    Block(Vec<block::BlockTransactionTrace>),
}

/// Responder type alias.
pub type Responder = oneshot::Sender<RpcResult<Response>>;
/// Debug requester type alias.
pub type DebugRequester =
    TracingUnboundedSender<((RequesterInput, Option<TraceParams>), Responder)>;

/// Debug.
pub struct Debug {
    /// Requester.
    pub requester: DebugRequester,
}

#[async_trait]
impl DebugServer for Debug {
    async fn trace_transaction(
        &self,
        transaction_hash: H256,
        params: Option<TraceParams>,
    ) -> RpcResult<single::TransactionTrace> {
        let requester = self.requester.clone();

        let (tx, rx) = oneshot::channel();
        // Send a message from the rpc handler to the service level task.
        requester
            .unbounded_send(((RequesterInput::Transaction(transaction_hash), params), tx))
            .map_err(|err| {
                internal_err(format!(
                    "failed to send request to debug service : {:?}",
                    err
                ))
            })?;

        // Receive a message from the service level task and send the rpc response.
        rx.await
            .map_err(|err| internal_err(format!("debug service dropped the channel : {:?}", err)))?
            .map(|res| match res {
                Response::Single(res) => res,
                _ => unreachable!(),
            })
    }

    async fn trace_block(
        &self,
        id: RequestBlockId,
        params: Option<TraceParams>,
    ) -> RpcResult<Vec<BlockTransactionTrace>> {
        let requester = self.requester.clone();

        let (tx, rx) = oneshot::channel();
        // Send a message from the rpc handler to the service level task.
        requester
            .unbounded_send(((RequesterInput::Block(id), params), tx))
            .map_err(|err| {
                internal_err(format!(
                    "failed to send request to debug service : {:?}",
                    err
                ))
            })?;

        // Receive a message from the service level task and send the rpc response.
        rx.await
            .map_err(|err| internal_err(format!("debug service dropped the channel : {:?}", err)))?
            .map(|res| match res {
                Response::Block(res) => res,
                _ => unreachable!(),
            })
    }

    async fn trace_call(
        &self,
        call_params: TraceCallParams,
        id: RequestBlockId,
        params: Option<TraceParams>,
    ) -> RpcResult<single::TransactionTrace> {
        let requester = self.requester.clone();

        let (tx, rx) = oneshot::channel();
        // Send a message from the rpc handler to the service level task.
        requester
            .unbounded_send(((RequesterInput::Call((id, call_params)), params), tx))
            .map_err(|err| {
                internal_err(format!(
                    "failed to send request to debug service : {:?}",
                    err
                ))
            })?;

        // Receive a message from the service level task and send the rpc response.
        rx.await
            .map_err(|err| internal_err(format!("debug service dropped the channel : {:?}", err)))?
            .map(|res| match res {
                Response::Single(res) => res,
                _ => unreachable!(),
            })
    }
}
