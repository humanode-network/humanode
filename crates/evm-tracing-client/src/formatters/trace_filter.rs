//! Trace filter formatter implementation.

use sp_core::H256;

use crate::listeners::call_list::Listener;
use crate::types::{
    block::{
        TransactionTrace, TransactionTraceAction, TransactionTraceOutput, TransactionTraceResult,
    },
    blockscout::BlockscoutCallInner as CallInner,
    CallResult, CreateResult, CreateType,
};

/// Trace filter formatter.
pub struct Formatter;

impl super::ResponseFormatter for Formatter {
    type Listener = Listener;
    type Response = Vec<TransactionTrace>;

    fn format(listener: Listener) -> Option<Vec<TransactionTrace>> {
        let mut traces = Vec::new();
        for (eth_tx_index, entry) in listener.entries.iter().enumerate() {
            // Skip empty BTreeMaps pushed to `entries`.
            // I.e. InvalidNonce or other pallet_evm::runner exits
            if entry.is_empty() {
                frame_support::log::debug!(
                    target: "tracing",
                    "Empty trace entry with transaction index {}, skipping...", eth_tx_index
                );
                continue;
            }
            let mut tx_traces: Vec<_> = entry
                .iter()
                .map(|(_, trace)| match trace.inner.clone() {
                    CallInner::Call {
                        input,
                        to,
                        res,
                        call_type,
                    } => TransactionTrace {
                        action: TransactionTraceAction::Call {
                            call_type,
                            from: trace.from,
                            gas: trace.gas,
                            input,
                            to,
                            value: trace.value,
                        },
                        // Can't be known here, must be inserted upstream.
                        block_hash: H256::default(),
                        // Can't be known here, must be inserted upstream.
                        block_number: 0,
                        output: match res {
                            CallResult::Output(output) => {
                                TransactionTraceOutput::Result(TransactionTraceResult::Call {
                                    gas_used: trace.gas_used,
                                    output,
                                })
                            }
                            CallResult::Error(error) => TransactionTraceOutput::Error(error),
                        },
                        subtraces: trace.subtraces,
                        trace_address: trace.trace_address.clone(),
                        // Can't be known here, must be inserted upstream.
                        transaction_hash: H256::default(),
                        // u32 (eth tx index) is big enough for this truncation to be practically impossible.
                        transaction_position: u32::try_from(eth_tx_index).unwrap(),
                    },
                    CallInner::Create { init, res } => {
                        TransactionTrace {
                            action: TransactionTraceAction::Create {
                                creation_method: CreateType::Create,
                                from: trace.from,
                                gas: trace.gas,
                                init,
                                value: trace.value,
                            },
                            // Can't be known here, must be inserted upstream.
                            block_hash: H256::default(),
                            // Can't be known here, must be inserted upstream.
                            block_number: 0,
                            output: match res {
                                CreateResult::Success {
                                    created_contract_address_hash,
                                    created_contract_code,
                                } => {
                                    TransactionTraceOutput::Result(TransactionTraceResult::Create {
                                        gas_used: trace.gas_used,
                                        code: created_contract_code,
                                        address: created_contract_address_hash,
                                    })
                                }
                                CreateResult::Error { error } => {
                                    TransactionTraceOutput::Error(error)
                                }
                            },
                            subtraces: trace.subtraces,
                            trace_address: trace.trace_address.clone(),
                            // Can't be known here, must be inserted upstream.
                            transaction_hash: H256::default(),
                            // u32 (eth tx index) is big enough for this truncation to be practically impossible.
                            transaction_position: u32::try_from(eth_tx_index).unwrap(),
                        }
                    }
                    CallInner::SelfDestruct { balance, to } => TransactionTrace {
                        action: TransactionTraceAction::Suicide {
                            address: trace.from,
                            balance,
                            refund_address: to,
                        },
                        // Can't be known here, must be inserted upstream.
                        block_hash: H256::default(),
                        // Can't be known here, must be inserted upstream.
                        block_number: 0,
                        output: TransactionTraceOutput::Result(TransactionTraceResult::Suicide),
                        subtraces: trace.subtraces,
                        trace_address: trace.trace_address.clone(),
                        // Can't be known here, must be inserted upstream.
                        transaction_hash: H256::default(),
                        // u32 (eth tx index) is big enough for this truncation to be practically impossible.
                        transaction_position: u32::try_from(eth_tx_index).unwrap(),
                    },
                })
                .collect();

            traces.append(&mut tx_traces);
        }
        Some(traces)
    }
}
