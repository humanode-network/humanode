//! Cache requester.

use sc_utils::mpsc::TracingUnboundedSender;
use sp_core::H256;
use tokio::sync::oneshot;
use tracing::instrument;

use super::TxsTraceRes;

/// An opaque batch ID.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheBatchId(pub u64);

/// Requests the cache task can accept.
pub enum CacheRequest {
    /// Request to start caching the provided range of blocks.
    /// The task will add to blocks to its pool and immediately return a new batch ID.
    StartBatch {
        /// Returns the ID of the batch for cancellation.
        sender: oneshot::Sender<CacheBatchId>,
        /// List of block hash to trace.
        blocks: Vec<H256>,
    },
    /// Fetch the traces for given block hash.
    /// The task will answer only when it has processed this block.
    GetTraces {
        /// Returns the array of traces or an error.
        sender: oneshot::Sender<TxsTraceRes>,
        /// Hash of the block.
        block: H256,
    },
    /// Notify the cache that it can stop the batch with that ID. Any block contained only in
    /// this batch and still not started will be discarded.
    StopBatch {
        /// Batch identifier.
        batch_id: CacheBatchId,
    },
}

/// Allows to interact with the cache task.
#[derive(Clone)]
pub struct CacheRequester(pub TracingUnboundedSender<CacheRequest>);

impl CacheRequester {
    /// Request to start caching the provided range of blocks.
    /// The task will add to blocks to its pool and immediately return the batch ID.
    #[instrument(skip(self))]
    pub async fn start_batch(&self, blocks: Vec<H256>) -> Result<CacheBatchId, String> {
        let (response_tx, response_rx) = oneshot::channel();
        let sender = self.0.clone();

        sender
            .unbounded_send(CacheRequest::StartBatch {
                sender: response_tx,
                blocks,
            })
            .map_err(|e| {
                format!(
                    "Failed to send request to the trace cache task. Error : {:?}",
                    e
                )
            })?;

        response_rx.await.map_err(|e| {
            format!(
                "Trace cache task closed the response channel. Error : {:?}",
                e
            )
        })
    }

    /// Fetch the traces for given block hash.
    /// The task will answer only when it has processed this block.
    /// The block should be part of a batch first. If no batch has requested the block it will
    /// return an error.
    #[instrument(skip(self))]
    pub async fn get_traces(&self, block: H256) -> TxsTraceRes {
        let (response_tx, response_rx) = oneshot::channel();
        let sender = self.0.clone();

        sender
            .unbounded_send(CacheRequest::GetTraces {
                sender: response_tx,
                block,
            })
            .map_err(|e| {
                format!(
                    "Failed to send request to the trace cache task. Error : {:?}",
                    e
                )
            })?;

        response_rx
            .await
            .map_err(|e| {
                format!(
                    "Trace cache task closed the response channel. Error : {:?}",
                    e
                )
            })?
            .map_err(|e| format!("Failed to replay block. Error : {:?}", e))
    }

    /// Notify the cache that it can stop the batch with that ID. Any block contained only in
    /// this batch and still in the waiting pool will be discarded.
    #[instrument(skip(self))]
    pub async fn stop_batch(&self, batch_id: CacheBatchId) {
        let sender = self.0.clone();

        // Here we don't care if the request has been accepted or refused, the caller can't
        // do anything with it.
        let _ = sender
            .unbounded_send(CacheRequest::StopBatch { batch_id })
            .map_err(|e| {
                format!(
                    "Failed to send request to the trace cache task. Error : {:?}",
                    e
                )
            });
    }
}
