//! Cache task.

use std::future::Future;
use std::{collections::BTreeMap, marker::PhantomData, sync::Arc, time::Duration};

use evm_tracing_api::EvmTracingApi;
use evm_tracing_client::{
    formatters::ResponseFormatter,
    types::block::{self, TransactionTrace},
};
use fc_rpc::OverrideHandle;
use fp_rpc::EthereumRuntimeRPCApi;
use futures::stream::FuturesUnordered;
use futures::{select, FutureExt, StreamExt};
use sc_client_api::{Backend, StateBackend, StorageProvider};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
    Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Header as HeaderT};
use substrate_prometheus_endpoint::{
    register, Counter, PrometheusError, Registry as PrometheusRegistry, U64,
};
use tokio::{
    sync::{mpsc, oneshot, Semaphore},
    time::sleep,
};
use tracing::{instrument, Instrument};

use super::{
    cache_requester::{CacheBatchId, CacheRequester},
    TxsTraceRes,
};
use crate::{trace::cache_requester::CacheRequest, types::TracerResponse};

/// Data stored for each block in the cache.
/// `active_batch_count` represents the number of batches using this
/// block. It will increase immediately when a batch is created, but will be
/// decrease only after the batch ends and its expiration delay passes.
/// It allows to keep the data in the cache for following requests that would use
/// this block, which is important to handle pagination efficiently.
struct CacheBlock {
    /// Active batch count.
    active_batch_count: usize,
    /// State.
    state: CacheBlockState,
}

/// State of a cached block. It can either be polled to be traced or cached.
enum CacheBlockState {
    /// Block has been added to the pool blocks to be replayed.
    /// It may be currently waiting to be replayed or being replayed.
    Pooled {
        /// Started flag.
        started: bool,
        /// Multiple requests might query the same block while it is pooled to be
        /// traced. They response channel is stored here, and the result will be
        /// sent in all of them when the tracing is finished.
        waiting_requests: Vec<oneshot::Sender<TxsTraceRes>>,
        /// Channel used to unqueue a tracing that has not yet started.
        /// A tracing will be unqueued if it has not yet been started and the last batch
        /// needing this block is ended (ignoring the expiration delay).
        /// It is not used directly, but dropping will wake up the receiver.
        #[allow(dead_code)]
        unqueue_sender: oneshot::Sender<()>,
    },
    /// Tracing has been completed and the result is available. No Runtime API call
    /// will be needed until this block cache is removed.
    Cached {
        /// Traces.
        traces: TxsTraceRes,
    },
}

/// Tracing a block is done in a separate tokio blocking task to avoid clogging the async threads.
/// For this reason a channel using this type is used by the blocking task to communicate with the
/// main cache task.
enum BlockingTaskMessage {
    /// Notify the tracing for this block has started as the blocking task got a permit from
    /// the semaphore. This is used to prevent the deletion of a cache entry for a block that has
    /// started being traced.
    Started {
        /// Block hash.
        block_hash: H256,
    },
    /// The tracing is finished and the result is sent to the main task.
    Finished {
        /// Block hash.
        block_hash: H256,
        /// Result.
        result: TxsTraceRes,
    },
}

/// Type wrapper for the cache task, generic over the Client, Block and Backend types.
pub struct CacheTask<B, C, BE> {
    /// Inner client.
    client: Arc<C>,
    /// Backend.
    backend: Arc<BE>,
    /// Blocking permits.
    blocking_permits: Arc<Semaphore>,
    /// Cached blocks.
    cached_blocks: BTreeMap<H256, CacheBlock>,
    /// Batches.
    batches: BTreeMap<u64, Vec<H256>>,
    /// Next batch id.
    next_batch_id: u64,
    /// Metrics.
    metrics: Option<Metrics>,
    /// Phantom data.
    _phantom: PhantomData<B>,
}

impl<B, C, BE> CacheTask<B, C, BE>
where
    BE: Backend<B> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<B>,
    C: StorageProvider<B, BE>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C::Api: BlockBuilder<B>,
    C::Api: EvmTracingApi<B>,
    C::Api: EthereumRuntimeRPCApi<B>,
    C::Api: ApiExt<B>,
{
    /// Create a new cache task.
    ///
    /// Returns a Future that needs to be added to a tokio executor, and a handle allowing to
    /// send requests to the task.
    pub fn create(
        client: Arc<C>,
        backend: Arc<BE>,
        cache_duration: Duration,
        blocking_permits: Arc<Semaphore>,
        overrides: Arc<OverrideHandle<B>>,
        prometheus: Option<PrometheusRegistry>,
    ) -> (impl Future<Output = ()>, CacheRequester) {
        let (requester_tx, mut requester_rx) =
            sc_utils::mpsc::tracing_unbounded("trace-filter-cache", 100_000);

        // Task running in the service.
        let task = async move {
			// The following variables are polled by the select! macro, and thus cannot be
			// part of Self without introducing borrowing issues.
			let mut batch_expirations = FuturesUnordered::new();
			let (blocking_tx, mut blocking_rx) =
				mpsc::channel(blocking_permits.available_permits().saturating_mul(2));
			let metrics = if let Some(registry) = prometheus {
				match Metrics::register(&registry) {
					Ok(metrics) => Some(metrics),
					Err(err) => {
						frame_support::log::error!(target: "tracing", "Failed to register metrics {err:?}");
						None
					}
				}
			} else {
				None
			};
			// Contains the inner state of the cache task, excluding the pooled futures/channels.
			// Having this object allows to refactor each event into its own function, simplifying
			// the main loop.
			let mut inner = Self {
				client,
				backend,
				blocking_permits,
				cached_blocks: BTreeMap::new(),
				batches: BTreeMap::new(),
				next_batch_id: 0,
				metrics,
				_phantom: Default::default(),
			};

			// Main event loop. This loop must not contain any direct .await, as we want to
			// react to events as fast as possible.
			loop {
				select! {
					request = requester_rx.next() => {
						match request {
							None => break,
							Some(CacheRequest::StartBatch {sender, blocks})
								=> inner.request_start_batch(&blocking_tx, sender, blocks, Arc::clone(&overrides)),
							Some(CacheRequest::GetTraces {sender, block})
								=> inner.request_get_traces(sender, block),
							Some(CacheRequest::StopBatch {batch_id}) => {
								batch_expirations.push(async move {
									sleep(cache_duration).await;
									batch_id
								});

								inner.request_stop_batch(batch_id);
							},
						}
					},
					message = blocking_rx.recv().fuse() => {
						match message {
							None => (),
							Some(BlockingTaskMessage::Started { block_hash })
								=> inner.blocking_started(block_hash),
							Some(BlockingTaskMessage::Finished { block_hash, result })
								=> inner.blocking_finished(block_hash, result),
						}
					},
					batch_id = batch_expirations.next() => {
						match batch_id {
							None => (),
							Some(batch_id) => inner.expired_batch(batch_id),
						}
					}
				}
			}
		}
		.instrument(tracing::debug_span!("trace_filter_cache"));

        (task, CacheRequester(requester_tx))
    }

    /// Handle the creation of a batch.
    /// Will start the tracing process for blocks that are not already in the cache.
    #[instrument(skip(self, blocking_tx, sender, blocks, overrides))]
    fn request_start_batch(
        &mut self,
        blocking_tx: &mpsc::Sender<BlockingTaskMessage>,
        sender: oneshot::Sender<CacheBatchId>,
        blocks: Vec<H256>,
        overrides: Arc<OverrideHandle<B>>,
    ) {
        tracing::trace!("Starting batch {}", self.next_batch_id);
        self.batches.insert(self.next_batch_id, blocks.clone());

        for block in blocks {
            // The block is already in the cache, awesome!
            if let Some(block_cache) = self.cached_blocks.get_mut(&block) {
                block_cache.active_batch_count = block_cache
                    .active_batch_count
                    .checked_add(1)
                    .expect("valid operation; qed");
                tracing::trace!(
                    "Cache hit for block {}, now used by {} batches.",
                    block,
                    block_cache.active_batch_count
                );
            }
            // Otherwise we need to queue this block for tracing.
            else {
                tracing::trace!("Cache miss for block {}, pooling it for tracing.", block);

                let blocking_permits = Arc::clone(&self.blocking_permits);
                let (unqueue_sender, unqueue_receiver) = oneshot::channel();
                let client = Arc::clone(&self.client);
                let backend = Arc::clone(&self.backend);
                let blocking_tx = blocking_tx.clone();
                let overrides = Arc::clone(&overrides);

                // Spawn all block caching asynchronously.
                // It will wait to obtain a permit, then spawn a blocking task.
                // When the blocking task returns its result, it is sent
                // thought a channel to the main task loop.
                tokio::spawn(
                    async move {
                        tracing::trace!("Waiting for blocking permit or task cancellation");
                        let _permit = select!(
                            _ = unqueue_receiver.fuse() => {
                            tracing::trace!("Tracing of the block has been cancelled.");
                                return;
                            },
                            permit = blocking_permits.acquire().fuse() => permit,
                        );

                        // Warn the main task that block tracing as started, and
                        // this block cache entry should not be removed.
                        let _ = blocking_tx
                            .send(BlockingTaskMessage::Started { block_hash: block })
                            .await;

                        tracing::trace!("Start block tracing in a blocking task.");

                        // Perform block tracing in a tokio blocking task.
                        let result = async {
                            tokio::task::spawn_blocking(move || {
                                Self::cache_block(client, backend, block, overrides)
                            })
                            .await
                            .map_err(|e| {
                                format!("Tracing Substrate block {} panicked : {:?}", block, e)
                            })?
                        }
                        .await
                        .map_err(|e| e.to_string());

                        tracing::trace!("Block tracing finished, sending result to main task.");

                        // Send a response to the main task.
                        let _ = blocking_tx
                            .send(BlockingTaskMessage::Finished {
                                block_hash: block,
                                result,
                            })
                            .await;
                    }
                    .instrument(tracing::trace_span!("Block tracing", block = %block)),
                );

                // Insert the block in the cache.
                self.cached_blocks.insert(
                    block,
                    CacheBlock {
                        active_batch_count: 1,
                        state: CacheBlockState::Pooled {
                            started: false,
                            waiting_requests: vec![],
                            unqueue_sender,
                        },
                    },
                );
            }
        }

        // Respond with the batch ID.
        let _ = sender.send(CacheBatchId(self.next_batch_id));

        // Increase batch ID for the next request.
        self.next_batch_id = self.next_batch_id.overflowing_add(1).0;
    }

    /// Handle a request to get the traces of the provided block.
    /// - If the result is stored in the cache, it sends it immediately.
    /// - If the block is currently being pooled, it is added to this block cache waiting list,
    ///   and all requests concerning this block will be satisfied when the tracing for this block
    ///   is finished.
    /// - If this block is missing from the cache, it means no batch asked for it. All requested
    ///   blocks should be contained in a batch beforehand, and thus an error is returned.
    #[instrument(skip(self))]
    fn request_get_traces(&mut self, sender: oneshot::Sender<TxsTraceRes>, block: H256) {
        if let Some(block_cache) = self.cached_blocks.get_mut(&block) {
            match &mut block_cache.state {
                CacheBlockState::Pooled {
                    ref mut waiting_requests,
                    ..
                } => {
                    tracing::warn!(
                        "A request asked a pooled block ({}), adding it to the list of waiting requests.",
                        block
                    );
                    waiting_requests.push(sender);
                    if let Some(metrics) = &self.metrics {
                        metrics.tracing_cache_misses.inc();
                    }
                }
                CacheBlockState::Cached { traces, .. } => {
                    tracing::warn!(
                        "A request asked a cached block ({}), sending the traces directly.",
                        block
                    );
                    let _ = sender.send(traces.clone());
                    if let Some(metrics) = &self.metrics {
                        metrics.tracing_cache_hits.inc();
                    }
                }
            }
        } else {
            tracing::warn!(
                "An RPC request asked to get a block ({}) which was not batched.",
                block
            );
            let _ = sender.send(Err(format!(
                "RPC request asked a block ({}) that was not batched",
                block
            )));
        }
    }

    /// Handle a request to stop a batch.
    /// For all blocks that needed to be traced, are only in this batch and not yet started, their
    /// tracing is cancelled to save CPU-time and avoid attacks requesting large amount of blocks.
    /// This batch data is not yet removed however. Instead a expiration delay timer is started
    /// after which the data will indeed be cleared. (the code for that is in the main loop code
    /// as it involved an unnamable type :C)
    #[instrument(skip(self))]
    fn request_stop_batch(&mut self, batch_id: CacheBatchId) {
        tracing::trace!("Stopping batch {}", batch_id.0);
        if let Some(blocks) = self.batches.get(&batch_id.0) {
            for block in blocks {
                let mut remove = false;

                // We remove early the block cache if this batch is the last
                // pooling this block.
                if let Some(block_cache) = self.cached_blocks.get_mut(block) {
                    if block_cache.active_batch_count == 1
                        && matches!(
                            block_cache.state,
                            CacheBlockState::Pooled { started: false, .. }
                        )
                    {
                        remove = true;
                    }
                }

                if remove {
                    tracing::trace!("Pooled block {} is no longer requested.", block);
                    // Remove block from the cache. Drops the value,
                    // closing all the channels contained in it.
                    let _ = self.cached_blocks.remove(block);
                }
            }
        }
    }

    /// A tracing blocking task notifies it got a permit and is starting the tracing.
    /// This started status is stored to avoid removing this block entry.
    #[instrument(skip(self))]
    fn blocking_started(&mut self, block_hash: H256) {
        if let Some(block_cache) = self.cached_blocks.get_mut(&block_hash) {
            if let CacheBlockState::Pooled {
                ref mut started, ..
            } = block_cache.state
            {
                *started = true;
            }
        }
    }

    /// A tracing blocking task notifies it has finished the tracing and provide the result.
    #[instrument(skip(self, result))]
    fn blocking_finished(&mut self, block_hash: H256, result: TxsTraceRes) {
        // In some cases it might be possible to receive traces of a block
        // that has no entry in the cache because it was removed of the pool
        // and received a permit concurrently. We just ignore it.
        if let Some(block_cache) = self.cached_blocks.get_mut(&block_hash) {
            if let CacheBlockState::Pooled {
                ref mut waiting_requests,
                ..
            } = block_cache.state
            {
                tracing::trace!(
                    "A new block ({}) has been traced, adding it to the cache and responding to {} waiting requests.",
                    block_hash,
                    waiting_requests.len()
                );
                // Send result in waiting channels.
                while let Some(channel) = waiting_requests.pop() {
                    let _ = channel.send(result.clone());
                }

                // Update cache entry.
                block_cache.state = CacheBlockState::Cached { traces: result };
            }
        }
    }

    /// A batch expiration delay timer has completed. It performs the cache cleaning for blocks
    /// not longer used by other batches.
    #[instrument(skip(self))]
    fn expired_batch(&mut self, batch_id: CacheBatchId) {
        if let Some(batch) = self.batches.remove(&batch_id.0) {
            for block in batch {
                // For each block of the batch, we remove it if it was the
                // last batch containing it.
                let mut remove = false;
                if let Some(block_cache) = self.cached_blocks.get_mut(&block) {
                    block_cache.active_batch_count = block_cache
                        .active_batch_count
                        .checked_sub(1)
                        .expect("valid operation; qed");

                    if block_cache.active_batch_count == 0 {
                        remove = true;
                    }
                }

                if remove {
                    let _ = self.cached_blocks.remove(&block);
                }
            }
        }
    }

    /// (In blocking task) Use the Runtime API to trace the block.
    #[instrument(skip(client, backend, overrides))]
    fn cache_block(
        client: Arc<C>,
        backend: Arc<BE>,
        substrate_hash: H256,
        overrides: Arc<OverrideHandle<B>>,
    ) -> TxsTraceRes {
        let api = client.runtime_api();
        let block_header = client
            .header(substrate_hash)
            .map_err(|e| {
                format!(
                    "Error when fetching substrate block {} header : {:?}",
                    substrate_hash, e
                )
            })?
            .ok_or_else(|| format!("Substrate block {} don't exist", substrate_hash))?;

        let height = *block_header.number();
        let substrate_parent_hash = *block_header.parent_hash();

        // Get Ethereum block data.
        let (eth_block, eth_transactions) = match (
            overrides
                .schemas
                .get(&fc_storage::onchain_storage_schema(
                    client.as_ref(),
                    substrate_hash,
                ))
                .unwrap_or(&overrides.fallback)
                .current_block(substrate_hash),
            overrides
                .fallback
                .current_transaction_statuses(substrate_hash),
        ) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                return Err(format!(
                    "Failed to get Ethereum block data for Substrate block {}",
                    substrate_hash
                ))
            }
        };

        let eth_block_hash = eth_block.header.hash();
        let eth_tx_hashes = eth_transactions
            .iter()
            .map(|t| t.transaction_hash)
            .collect();

        // Get extrinsics (containing Ethereum ones).
        let extrinsics = backend
            .blockchain()
            .body(substrate_hash)
            .map_err(|e| {
                format!(
                    "Blockchain error when fetching extrinsics of block {} : {:?}",
                    height, e
                )
            })?
            .ok_or_else(|| format!("Could not find block {} when fetching extrinsics.", height))?;

        // Trace the block.
        let f = || -> Result<_, String> {
            let result = api.trace_block(
                substrate_parent_hash,
                extrinsics,
                eth_tx_hashes,
                &block_header,
            );

            result
                .map_err(|e| format!("Blockchain error when replaying block {} : {:?}", height, e))?
                .map_err(|e| {
                    tracing::warn!(
                        target: "tracing",
                        "Internal runtime error when replaying block {} : {:?}",
                        height,
                        e
                    );
                    format!(
                        "Internal runtime error when replaying block {} : {:?}",
                        height, e
                    )
                })?;

            Ok(TracerResponse::Block)
        };

        let eth_transactions_by_index: BTreeMap<u32, H256> = eth_transactions
            .iter()
            .map(|t| (t.transaction_index, t.transaction_hash))
            .collect();

        let mut proxy = evm_tracing_client::listeners::call_list::Listener::default();
        proxy.using(f)?;

        let traces: Vec<TransactionTrace> =
            evm_tracing_client::formatters::trace_filter::Formatter::format(proxy)
                .ok_or("Fail to format proxy")?
                .into_iter()
                .filter_map(|mut trace| {
                    match eth_transactions_by_index.get(&trace.transaction_position) {
                        Some(transaction_hash) => {
                            trace.block_hash = eth_block_hash;
                            trace.block_number = height;
                            trace.transaction_hash = *transaction_hash;

                            // Reformat error messages.
                            if let block::TransactionTraceOutput::Error(ref mut error) =
                                trace.output
                            {
                                if error.as_slice() == b"execution reverted" {
                                    *error = b"Reverted".to_vec();
                                }
                            }
                            Some(trace)
                        }
                        None => {
                            frame_support::log::warn!(
                                target: "tracing",
                                "A trace in block {} does not map to any known ethereum transaction. Trace: {:?}",
                                height,
                                trace,
                            );

                            None
                        }
                    }
                })
                .collect();

        Ok(traces)
    }
}

/// Prometheus metrics for tracing.
#[derive(Clone)]
pub struct Metrics {
    /// Tracing cache hits.
    tracing_cache_hits: Counter<U64>,
    /// Tracing cache misses.
    tracing_cache_misses: Counter<U64>,
}

impl Metrics {
    /// Register.
    pub fn register(registry: &PrometheusRegistry) -> Result<Self, PrometheusError> {
        Ok(Self {
            tracing_cache_hits: register(
                Counter::new("tracing_cache_hits", "Number of tracing cache hits.")?,
                registry,
            )?,
            tracing_cache_misses: register(
                Counter::new("tracing_cache_misses", "Number of tracing cache misses.")?,
                registry,
            )?,
        })
    }
}
