//! Trace related implementation.

use core::{FilterRequest, TraceServer};
use std::{marker::PhantomData, sync::Arc};

use cache_requester::CacheRequester;
use evm_tracing_client::types::block::{self, TransactionTrace};
use jsonrpsee::core::{async_trait, RpcResult};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_core::H256;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

use crate::types::{RequestBlockId, RequestBlockTag};

pub mod cache_requester;
pub mod cache_task;
pub mod core;

/// Transaction trace result alias.
type TxsTraceRes = Result<Vec<TransactionTrace>, String>;

/// RPC handler. Will communicate with a `CacheTask` through a `CacheRequester`.
pub struct Trace<B, C> {
    /// Inner client.
    client: Arc<C>,
    /// Cache requester.
    requester: CacheRequester,
    /// Max count.
    max_count: u32,
    /// Phantom data.
    _phantom: PhantomData<B>,
}

impl<B, C> Clone for Trace<B, C> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            requester: self.requester.clone(),
            max_count: self.max_count,
            _phantom: PhantomData,
        }
    }
}

impl<B, C> Trace<B, C>
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
{
    /// Create a new RPC handler.
    pub fn new(client: Arc<C>, requester: CacheRequester, max_count: u32) -> Self {
        Self {
            client,
            requester,
            max_count,
            _phantom: PhantomData,
        }
    }

    /// Convert an optional block ID (number or tag) to a block height.
    fn block_id(&self, id: Option<RequestBlockId>) -> Result<u32, &'static str> {
        match id {
            Some(RequestBlockId::Number(n)) => Ok(n),
            None | Some(RequestBlockId::Tag(RequestBlockTag::Latest)) => {
                Ok(self.client.info().best_number)
            }
            Some(RequestBlockId::Tag(RequestBlockTag::Earliest)) => Ok(0),
            Some(RequestBlockId::Tag(RequestBlockTag::Pending)) => {
                Err("'pending' is not supported")
            }
            Some(RequestBlockId::Hash(_)) => Err("Block hash not supported"),
        }
    }

    /// `trace_filter` endpoint (wrapped in the trait implementation with futures compatibility).
    async fn filter(self, req: FilterRequest) -> TxsTraceRes {
        let from_block = self.block_id(req.from_block)?;
        let to_block = self.block_id(req.to_block)?;
        let block_heights = from_block..=to_block;

        let count = req.count.unwrap_or(self.max_count);
        if count > self.max_count {
            return Err(format!(
                "count ({}) can't be greater than maximum ({})",
                count, self.max_count
            ));
        }

        // Build a list of all the Substrate block hashes that need to be traced.
        let mut block_hashes = vec![];
        for block_height in block_heights {
            if block_height == 0 {
                continue; // no traces for genesis block.
            }

            let block_hash = self
                .client
                .hash(block_height)
                .map_err(|e| {
                    format!(
                        "Error when fetching block {} header : {:?}",
                        block_height, e
                    )
                })?
                .ok_or_else(|| format!("Block with height {} don't exist", block_height))?;

            block_hashes.push(block_hash);
        }

        // Start a batch with these blocks.
        let batch_id = self.requester.start_batch(block_hashes.clone()).await?;
        // Fetch all the traces. It is done in another function to simplify error handling and allow
        // to call the following `stop_batch` regardless of the result. This is important for the
        // cache cleanup to work properly.
        let res = self.fetch_traces(req, &block_hashes, count as usize).await;
        // Stop the batch, allowing the cache task to remove useless non-started block traces and
        // start the expiration delay.
        self.requester.stop_batch(batch_id).await;

        res
    }

    /// Fetch traces.
    async fn fetch_traces(
        &self,
        req: FilterRequest,
        block_hashes: &[H256],
        count: usize,
    ) -> TxsTraceRes {
        let from_address = req.from_address.unwrap_or_default();
        let to_address = req.to_address.unwrap_or_default();

        let mut traces = vec![];

        for &block_hash in block_hashes {
            // Request the traces of this block to the cache service.
            // This will resolve quickly if the block is already cached, or wait until the block
            // has finished tracing.
            let block_traces = self.requester.get_traces(block_hash).await?;

            // Filter addresses.
            let mut block_traces: Vec<_> = block_traces
                .iter()
                .filter(|trace| match trace.action {
                    block::TransactionTraceAction::Call { from, to, .. } => {
                        (from_address.is_empty() || from_address.contains(&from))
                            && (to_address.is_empty() || to_address.contains(&to))
                    }
                    block::TransactionTraceAction::Create { from, .. } => {
                        (from_address.is_empty() || from_address.contains(&from))
                            && to_address.is_empty()
                    }
                    block::TransactionTraceAction::Suicide { address, .. } => {
                        (from_address.is_empty() || from_address.contains(&address))
                            && to_address.is_empty()
                    }
                })
                .cloned()
                .collect();

            // Don't insert anything if we're still before "after".
            if let Some(traces_amount) = block_traces.len().checked_sub(
                // usize is big enough for this overflow to be practically impossible.
                req.after.unwrap_or(0).try_into().unwrap(),
            ) {
                // If the current Vec of traces is across the "after" marker,
                // we skip some elements of it.
                if let Some(skip) = block_traces.len().checked_sub(traces_amount) {
                    block_traces = block_traces.into_iter().skip(skip).collect();
                }

                traces.append(&mut block_traces);

                // If we go over "count" (the limit), we trim and exit the loop,
                // unless we used the default maximum, in which case we return an error.
                if traces_amount >= count {
                    if req.count.is_none() {
                        return Err(format!(
                            "the amount of traces goes over the maximum ({}), please use 'after' and 'count' in your request",
                            self.max_count
                        ));
                    }

                    traces = traces.into_iter().take(count).collect();
                    break;
                }
            }
        }

        Ok(traces)
    }
}

#[async_trait]
impl<B, C> TraceServer for Trace<B, C>
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
{
    async fn filter(&self, filter: FilterRequest) -> RpcResult<Vec<TransactionTrace>> {
        self.clone()
            .filter(filter)
            .await
            .map_err(fc_rpc::internal_err)
    }
}
