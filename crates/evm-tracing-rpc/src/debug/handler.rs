//! Debug handler implementation.

use std::{collections::BTreeMap, future::Future, marker::PhantomData, sync::Arc};

use evm_tracing_api::EvmTracingApi;
use evm_tracing_client::{
    formatters::ResponseFormatter,
    types::{
        block::BlockTransactionTrace,
        call_tracer::CallTracerInner,
        single::{self, TransactionTrace},
    },
};
use fc_rpc::{frontier_backend_client, internal_err, OverrideHandle};
use fp_rpc::EthereumRuntimeRPCApi;
use futures::StreamExt;
use jsonrpsee::core::RpcResult;
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
    Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_core::{H160, H256};
use sp_runtime::{
    generic::BlockId,
    traits::{BlakeTwo256, Block as BlockT, Header as HeaderT, UniqueSaturatedInto},
};
use tokio::sync::Semaphore;

use super::{
    core::{TraceCallParams, TraceParams},
    DebugRequester, RequesterInput, Response,
};
use crate::types::{RequestBlockId, RequestBlockTag, TracerInput, TracerResponse};

/// Debug handler.
pub struct DebugHandler<B: BlockT, C, BE>(PhantomData<(B, C, BE)>);

impl<B, C, BE> DebugHandler<B, C, BE>
where
    BE: Backend<B> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<B>,
    C: StorageProvider<B, BE>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
    C: Send + Sync + 'static,
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    C::Api: BlockBuilder<B>,
    C::Api: EvmTracingApi<B>,
    C::Api: EthereumRuntimeRPCApi<B>,
    C::Api: ApiExt<B>,
{
    /// Task spawned at service level that listens for messages on the rpc channel and spawns
    /// blocking tasks using a permit pool.
    pub fn task(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<dyn fc_db::BackendReader<B> + Send + Sync>,
        permit_pool: Arc<Semaphore>,
        overrides: Arc<OverrideHandle<B>>,
        raw_max_memory_usage: usize,
    ) -> (impl Future<Output = ()>, DebugRequester) {
        let (tx, mut rx): (DebugRequester, _) =
            sc_utils::mpsc::tracing_unbounded("debug-requester", 100_000);

        let fut = async move {
            loop {
                match rx.next().await {
                    Some((
                        (RequesterInput::Transaction(transaction_hash), params),
                        response_tx,
                    )) => {
                        let client = Arc::clone(&client);
                        let backend = Arc::clone(&backend);
                        let frontier_backend = Arc::clone(&frontier_backend);
                        let permit_pool = Arc::clone(&permit_pool);
                        let overrides = Arc::clone(&overrides);

                        tokio::task::spawn(async move {
                            let _ = response_tx.send(
                                async {
                                    let _permit = permit_pool.acquire().await;
                                    tokio::task::spawn_blocking(move || {
                                        Self::handle_transaction_request(
                                            client,
                                            backend,
                                            frontier_backend,
                                            transaction_hash,
                                            params,
                                            overrides,
                                            raw_max_memory_usage,
                                        )
                                    })
                                    .await
                                    .map_err(|e| {
                                        internal_err(format!(
                                            "Internal error on spawned task : {:?}",
                                            e
                                        ))
                                    })?
                                }
                                .await,
                            );
                        });
                    }
                    Some((
                        (RequesterInput::Call((request_block_id, call_params)), params),
                        response_tx,
                    )) => {
                        let client = Arc::clone(&client);
                        let frontier_backend = Arc::clone(&frontier_backend);
                        let permit_pool = Arc::clone(&permit_pool);

                        tokio::task::spawn(async move {
                            let _ = response_tx.send(
                                async {
                                    let _permit = permit_pool.acquire().await;
                                    tokio::task::spawn_blocking(move || {
                                        Self::handle_call_request(
                                            client,
                                            frontier_backend,
                                            request_block_id,
                                            call_params,
                                            params,
                                            raw_max_memory_usage,
                                        )
                                    })
                                    .await
                                    .map_err(|e| {
                                        internal_err(format!(
                                            "Internal error on spawned task : {:?}",
                                            e
                                        ))
                                    })?
                                }
                                .await,
                            );
                        });
                    }
                    Some(((RequesterInput::Block(request_block_id), params), response_tx)) => {
                        let client = Arc::clone(&client);
                        let backend = Arc::clone(&backend);
                        let frontier_backend = Arc::clone(&frontier_backend);
                        let permit_pool = Arc::clone(&permit_pool);
                        let overrides = Arc::clone(&overrides);

                        tokio::task::spawn(async move {
                            let _ = response_tx.send(
                                async {
                                    let _permit = permit_pool.acquire().await;

                                    tokio::task::spawn_blocking(move || {
                                        Self::handle_block_request(
                                            client,
                                            backend,
                                            frontier_backend,
                                            request_block_id,
                                            params,
                                            overrides,
                                        )
                                    })
                                    .await
                                    .map_err(|e| {
                                        internal_err(format!(
                                            "Internal error on spawned task : {:?}",
                                            e
                                        ))
                                    })?
                                }
                                .await,
                            );
                        });
                    }
                    _ => {}
                }
            }
        };
        (fut, tx)
    }

    /// Handle params.
    fn handle_params(params: Option<TraceParams>) -> RpcResult<(TracerInput, single::TraceType)> {
        // Set trace input and type
        match params {
            Some(TraceParams {
                tracer: Some(tracer),
                ..
            }) => {
                /// Blockscout related js code hash.
                const BLOCKSCOUT_JS_CODE_HASH: [u8; 16] =
                    hex_literal::hex!("94d9f08796f91eb13a2e82a6066882f7");
                /// Blockscout V2 related js code hash.
                const BLOCKSCOUT_JS_CODE_HASH_V2: [u8; 16] =
                    hex_literal::hex!("89db13694675692951673a1e6e18ff02");
                let hash = sp_io::hashing::twox_128(tracer.as_bytes());
                let tracer =
                    if hash == BLOCKSCOUT_JS_CODE_HASH || hash == BLOCKSCOUT_JS_CODE_HASH_V2 {
                        Some(TracerInput::Blockscout)
                    } else if tracer == "callTracer" {
                        Some(TracerInput::CallTracer)
                    } else {
                        None
                    };
                if let Some(tracer) = tracer {
                    Ok((tracer, single::TraceType::CallList))
                } else {
                    Err(internal_err(format!(
                        "javascript based tracing is not available (hash :{:?})",
                        hash
                    )))
                }
            }
            Some(params) => Ok((
                TracerInput::None,
                single::TraceType::Raw {
                    disable_storage: params.disable_storage.unwrap_or(false),
                    disable_memory: params.disable_memory.unwrap_or(false),
                    disable_stack: params.disable_stack.unwrap_or(false),
                },
            )),
            _ => Ok((
                TracerInput::None,
                single::TraceType::Raw {
                    disable_storage: false,
                    disable_memory: false,
                    disable_stack: false,
                },
            )),
        }
    }

    /// Handle block request.
    fn handle_block_request(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<dyn fc_db::BackendReader<B> + Send + Sync>,
        request_block_id: RequestBlockId,
        params: Option<TraceParams>,
        overrides: Arc<OverrideHandle<B>>,
    ) -> RpcResult<Response> {
        let (tracer_input, trace_type) = Self::handle_params(params)?;

        let reference_id: BlockId<B> = match request_block_id {
            RequestBlockId::Number(n) => Ok(BlockId::Number(n.unique_saturated_into())),
            RequestBlockId::Tag(RequestBlockTag::Latest) => {
                Ok(BlockId::Number(client.info().best_number))
            }
            RequestBlockId::Tag(RequestBlockTag::Earliest) => {
                Ok(BlockId::Number(0u32.unique_saturated_into()))
            }
            RequestBlockId::Tag(RequestBlockTag::Pending) => {
                Err(internal_err("'pending' blocks are not supported"))
            }
            RequestBlockId::Hash(eth_hash) => {
                match futures::executor::block_on(frontier_backend_client::load_hash::<B, C>(
                    client.as_ref(),
                    frontier_backend.as_ref(),
                    eth_hash,
                )) {
                    Ok(Some(hash)) => Ok(BlockId::Hash(hash)),
                    Ok(_) => Err(internal_err("Block hash not found".to_string())),
                    Err(e) => Err(e),
                }
            }
        }?;

        // Get `ApiRef`. This handle allows to keep changes between txs in an internal buffer.
        let api = client.runtime_api();

        // Get Blockchain backend
        let blockchain = backend.blockchain();
        // Get the header I want to work with.
        let Ok(hash) = client.expect_block_hash_from_id(&reference_id) else {
            return Err(internal_err("Block header not found"));
        };
        let header = match client.header(hash) {
            Ok(Some(h)) => h,
            _ => return Err(internal_err("Block header not found")),
        };

        // Get parent blockid.
        let parent_block_hash = *header.parent_hash();

        let statuses = overrides
            .fallback
            .current_transaction_statuses(hash)
            .unwrap_or_default();

        /// Partial ethereum transaction data to check if a trace match an ethereum transaction.
        struct EthTxPartial {
            /// Transaction hash.
            transaction_hash: H256,
            /// From address.
            from: H160,
            /// To address.
            to: Option<H160>,
        }

        // Known ethereum transaction hashes.
        let eth_transactions_by_index: BTreeMap<u32, EthTxPartial> = statuses
            .iter()
            .map(|status| {
                (
                    status.transaction_index,
                    EthTxPartial {
                        transaction_hash: status.transaction_hash,
                        from: status.from,
                        to: status.to,
                    },
                )
            })
            .collect();

        let eth_tx_hashes: Vec<_> = eth_transactions_by_index
            .values()
            .map(|tx| tx.transaction_hash)
            .collect();

        // If there are no ethereum transactions in the block return empty trace right away.
        if eth_tx_hashes.is_empty() {
            return Ok(Response::Block(vec![]));
        }

        // Get block extrinsics.
        let exts = blockchain
            .body(hash)
            .map_err(|e| internal_err(format!("Fail to read blockchain db: {:?}", e)))?
            .unwrap_or_default();

        // Trace the block.
        let f = || -> RpcResult<_> {
            let result = api.trace_block(parent_block_hash, exts, eth_tx_hashes, &header);

            result
                .map_err(|e| {
                    internal_err(format!(
                        "Blockchain error when replaying block {} : {:?}",
                        reference_id, e
                    ))
                })?
                .map_err(|e| {
                    internal_err(format!(
                        "Internal runtime error when replaying block {} : {:?}",
                        reference_id, e
                    ))
                })?;

            Ok(TracerResponse::Block)
        };

        // Offset to account for old buggy transactions that are in trace not in the ethereum block.
        let mut tx_position_offset = 0;

        match trace_type {
            single::TraceType::CallList => {
                let mut proxy = evm_tracing_client::listeners::call_list::Listener::default();
                proxy.using(f)?;
                proxy.finish_transaction();
                let response = match tracer_input {
                    TracerInput::CallTracer => {
                        let result =
                            evm_tracing_client::formatters::call_tracer::Formatter::format(proxy)
                                .ok_or("Trace result is empty.")
                                .map_err(|e| internal_err(format!("{:?}", e)))?
                                .into_iter()
                                .filter_map(|mut trace: BlockTransactionTrace| {
                                    if let Some(EthTxPartial {
                                        transaction_hash,
                                        from,
                                        to,
                                    }) = eth_transactions_by_index.get(
                                        &(trace.tx_position.checked_sub(tx_position_offset))
                                            .expect("valid operation; qed"),
                                    ) {
                                        // Verify that the trace matches the ethereum transaction.
                                        let (trace_from, trace_to) = match trace.result {
                                            TransactionTrace::Raw { .. }
                                            | TransactionTrace::CallList(_) => {
                                                (Default::default(), None)
                                            }
                                            TransactionTrace::CallListNested(ref call) => {
                                                match call {
                                                    single::Call::Blockscout(_) => {
                                                        (Default::default(), None)
                                                    }
                                                    single::Call::CallTracer(call) => (
                                                        call.from,
                                                        match call.inner {
                                                            CallTracerInner::Call {
                                                                to, ..
                                                            } => Some(to),
                                                            CallTracerInner::Create { .. }
                                                            | CallTracerInner::SelfDestruct {
                                                                ..
                                                            } => None,
                                                        },
                                                    ),
                                                }
                                            }
                                        };
                                        if trace_from == *from && trace_to == *to {
                                            trace.tx_hash = *transaction_hash;
                                            Some(trace)
                                        } else {
                                            // If the trace does not match the ethereum transaction
                                            // it means that the trace is about a buggy transaction that is not in the block
                                            // we need to offset the tx_position.
                                            tx_position_offset = tx_position_offset
                                                .checked_add(1)
                                                .expect("valid operation; qed");

                                            None
                                        }
                                    } else {
                                        // If the transaction is not in the ethereum block
                                        // it should not appear in the block trace.
                                        tx_position_offset = tx_position_offset
                                            .checked_add(1)
                                            .expect("valid operation; qed");

                                        None
                                    }
                                })
                                .collect::<Vec<BlockTransactionTrace>>();

                        let n_txs = eth_transactions_by_index.len();
                        let n_traces = result.len();
                        if n_txs != n_traces {
                            frame_support::log::warn!(
								"The traces in block {:?} don't match with the number of ethereum transactions. (txs: {}, traces: {})",
								request_block_id,
								n_txs,
								n_traces
							);
                        }

                        Ok(result)
                    }
                    _ => Err(internal_err(
                        "Bug: failed to resolve the tracer format.".to_string(),
                    )),
                }?;

                Ok(Response::Block(response))
            }
            _ => Err(internal_err(
                "debug_traceBlock functions currently only support callList mode (enabled
				by providing `{{'tracer': 'callTracer'}}` in the request)."
                    .to_string(),
            )),
        }
    }

    /// Replays a transaction in the Runtime at a given block height.
    ///
    /// In order to successfully reproduce the result of the original transaction we need a correct
    /// state to replay over.
    ///
    /// Substrate allows to apply extrinsics in the Runtime and thus creating an overlaid state.
    /// These overlaid changes will live in-memory for the lifetime of the `ApiRef`.
    fn handle_transaction_request(
        client: Arc<C>,
        backend: Arc<BE>,
        frontier_backend: Arc<dyn fc_db::BackendReader<B> + Send + Sync>,
        transaction_hash: H256,
        params: Option<TraceParams>,
        overrides: Arc<OverrideHandle<B>>,
        raw_max_memory_usage: usize,
    ) -> RpcResult<Response> {
        let (tracer_input, trace_type) = Self::handle_params(params)?;

        let (hash, index) =
            match futures::executor::block_on(frontier_backend_client::load_transactions::<B, C>(
                client.as_ref(),
                frontier_backend.as_ref(),
                transaction_hash,
                false,
            )) {
                Ok(Some((hash, index))) => (hash, index as usize),
                Ok(None) => return Err(internal_err("Transaction hash not found".to_string())),
                Err(e) => return Err(e),
            };

        let reference_id =
            match futures::executor::block_on(frontier_backend_client::load_hash::<B, C>(
                client.as_ref(),
                frontier_backend.as_ref(),
                hash,
            )) {
                Ok(Some(hash)) => BlockId::Hash(hash),
                Ok(_) => return Err(internal_err("Block hash not found".to_string())),
                Err(e) => return Err(e),
            };
        // Get `ApiRef`. This handle allow to keep changes between txs in an internal buffer.
        let api = client.runtime_api();

        // Get blockchain backend.
        let blockchain = backend.blockchain();
        // Get the header I want to work with.
        let Ok(reference_hash) = client.expect_block_hash_from_id(&reference_id) else {
            return Err(internal_err("Block header not found"));
        };
        let header = match client.header(reference_hash) {
            Ok(Some(h)) => h,
            _ => return Err(internal_err("Block header not found")),
        };
        // Get parent blockid.
        let parent_block_hash = *header.parent_hash();

        // Get block extrinsics.
        let exts = blockchain
            .body(reference_hash)
            .map_err(|e| internal_err(format!("Fail to read blockchain db: {:?}", e)))?
            .unwrap_or_default();

        let reference_block = overrides
            .schemas
            .get(&fc_storage::onchain_storage_schema(
                client.as_ref(),
                reference_hash,
            ))
            .unwrap_or(&overrides.fallback)
            .current_block(reference_hash);

        // Get the actual ethereum transaction.
        if let Some(block) = reference_block {
            let transactions = block.transactions;
            if let Some(transaction) = transactions.get(index) {
                let f = || -> RpcResult<_> {
                    let result =
                        api.trace_transaction(parent_block_hash, exts, transaction, &header);

                    result
                        .map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?
                        .map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?;

                    Ok(TracerResponse::Single)
                };

                return match trace_type {
                    single::TraceType::Raw {
                        disable_storage,
                        disable_memory,
                        disable_stack,
                    } => {
                        let mut proxy = evm_tracing_client::listeners::raw::Listener::new(
                            disable_storage,
                            disable_memory,
                            disable_stack,
                            raw_max_memory_usage,
                        );
                        proxy.using(f)?;
                        Ok(Response::Single(
                            evm_tracing_client::formatters::raw::Formatter::format(proxy).ok_or(
                                internal_err(
                                    "replayed transaction generated too much data. \
								try disabling memory or storage?",
                                ),
                            )?,
                        ))
                    }
                    single::TraceType::CallList => {
                        let mut proxy =
                            evm_tracing_client::listeners::call_list::Listener::default();
                        proxy.using(f)?;
                        proxy.finish_transaction();
                        let response = match tracer_input {
                            TracerInput::Blockscout => {
                                evm_tracing_client::formatters::blockscout::Formatter::format(proxy)
                                    .ok_or("Trace result is empty.")
                                    .map_err(|e| internal_err(format!("{:?}", e)))
                            }
                            TracerInput::CallTracer => {
                                let mut res =
                                    evm_tracing_client::formatters::call_tracer::Formatter::format(
                                        proxy,
                                    )
                                    .ok_or("Trace result is empty.")
                                    .map_err(|e| internal_err(format!("{:?}", e)))?;
                                Ok(res.pop().expect("Trace result is empty.").result)
                            }
                            _ => Err(internal_err(
                                "Bug: failed to resolve the tracer format.".to_string(),
                            )),
                        }?;
                        Ok(Response::Single(response))
                    }
                    not_supported => Err(internal_err(format!(
                        "Bug: `handle_transaction_request` does not support {:?}.",
                        not_supported
                    ))),
                };
            }
        }
        Err(internal_err("Runtime block call failed".to_string()))
    }

    /// Handle call request.
    fn handle_call_request(
        client: Arc<C>,
        frontier_backend: Arc<dyn fc_db::BackendReader<B> + Send + Sync>,
        request_block_id: RequestBlockId,
        call_params: TraceCallParams,
        trace_params: Option<TraceParams>,
        raw_max_memory_usage: usize,
    ) -> RpcResult<Response> {
        let (tracer_input, trace_type) = Self::handle_params(trace_params)?;

        let reference_id: BlockId<B> = match request_block_id {
            RequestBlockId::Number(n) => Ok(BlockId::Number(n.unique_saturated_into())),
            RequestBlockId::Tag(RequestBlockTag::Latest) => {
                Ok(BlockId::Number(client.info().best_number))
            }
            RequestBlockId::Tag(RequestBlockTag::Earliest) => {
                Ok(BlockId::Number(0u32.unique_saturated_into()))
            }
            RequestBlockId::Tag(RequestBlockTag::Pending) => {
                Err(internal_err("'pending' blocks are not supported"))
            }
            RequestBlockId::Hash(eth_hash) => {
                match futures::executor::block_on(frontier_backend_client::load_hash::<B, C>(
                    client.as_ref(),
                    frontier_backend.as_ref(),
                    eth_hash,
                )) {
                    Ok(Some(hash)) => Ok(BlockId::Hash(hash)),
                    Ok(_) => Err(internal_err("Block hash not found".to_string())),
                    Err(e) => Err(e),
                }
            }
        }?;

        let api = client.runtime_api();

        // Get the header I want to work with.
        let Ok(hash) = client.expect_block_hash_from_id(&reference_id) else {
            return Err(internal_err("Block header not found"));
        };
        let header = match client.header(hash) {
            Ok(Some(h)) => h,
            _ => return Err(internal_err("Block header not found")),
        };
        // Get parent blockid.
        let parent_block_hash = *header.parent_hash();

        let TraceCallParams {
            from,
            to,
            gas_price,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            gas,
            value,
            data,
            nonce,
            access_list,
            ..
        } = call_params;

        let (max_fee_per_gas, max_priority_fee_per_gas) =
            match (gas_price, max_fee_per_gas, max_priority_fee_per_gas) {
                (gas_price, None, None) => {
                    // Legacy request, all default to gas price. A zero-set gas price is None.
                    let gas_price = if gas_price.unwrap_or_default().is_zero() {
                        None
                    } else {
                        gas_price
                    };
                    (gas_price, gas_price)
                }
                (_, max_fee, max_priority) => {
                    // EIP-1559: A zero-set max fee is None.
                    let max_fee = if max_fee.unwrap_or_default().is_zero() {
                        None
                    } else {
                        max_fee
                    };
                    // Ensure `max_priority_fee_per_gas` is less or equal to `max_fee_per_gas`.
                    if let Some(max_priority) = max_priority {
                        let max_fee = max_fee.unwrap_or_default();
                        if max_priority > max_fee {
                            return Err(internal_err(
							"Invalid input: `max_priority_fee_per_gas` greater than `max_fee_per_gas`",
						));
                        }
                    }
                    (max_fee, max_priority)
                }
            };

        let gas_limit = match gas {
            Some(amount) => amount,
            None => {
                if let Some(block) = api
                    .current_block(parent_block_hash)
                    .map_err(|err| internal_err(format!("runtime error: {:?}", err)))?
                {
                    block.header.gas_limit
                } else {
                    return Err(internal_err(
                        "block unavailable, cannot query gas limit".to_string(),
                    ));
                }
            }
        };
        let data = data.map(|d| d.0).unwrap_or_default();

        let access_list = access_list.unwrap_or_default();

        let f = || -> RpcResult<_> {
            api.trace_call(
                parent_block_hash,
                &header,
                from.unwrap_or_default(),
                to,
                data,
                value.unwrap_or_default(),
                gas_limit,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                Some(
                    access_list
                        .into_iter()
                        .map(|item| (item.address, item.storage_keys))
                        .collect(),
                ),
            )
            .map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?
            .map_err(|e| internal_err(format!("DispatchError: {:?}", e)))?;

            Ok(TracerResponse::Single)
        };

        match trace_type {
            single::TraceType::Raw {
                disable_storage,
                disable_memory,
                disable_stack,
            } => {
                let mut proxy = evm_tracing_client::listeners::raw::Listener::new(
                    disable_storage,
                    disable_memory,
                    disable_stack,
                    raw_max_memory_usage,
                );
                proxy.using(f)?;
                Ok(Response::Single(
                    evm_tracing_client::formatters::raw::Formatter::format(proxy).ok_or(
                        internal_err(
                            "replayed transaction generated too much data. \
						try disabling memory or storage?",
                        ),
                    )?,
                ))
            }
            single::TraceType::CallList => {
                let mut proxy = evm_tracing_client::listeners::call_list::Listener::default();
                proxy.using(f)?;
                proxy.finish_transaction();
                let response = match tracer_input {
                    TracerInput::Blockscout => {
                        evm_tracing_client::formatters::blockscout::Formatter::format(proxy)
                            .ok_or("Trace result is empty.")
                            .map_err(|e| internal_err(format!("{:?}", e)))
                    }
                    TracerInput::CallTracer => {
                        let mut res =
                            evm_tracing_client::formatters::call_tracer::Formatter::format(proxy)
                                .ok_or("Trace result is empty.")
                                .map_err(|e| internal_err(format!("{:?}", e)))?;
                        Ok(res.pop().expect("Trace result is empty.").result)
                    }
                    _ => Err(internal_err(
                        "Bug: failed to resolve the tracer format.".to_string(),
                    )),
                }?;
                Ok(Response::Single(response))
            }
            not_supported => Err(internal_err(format!(
                "Bug: `handle_call_request` does not support {:?}.",
                not_supported
            ))),
        }
    }
}
