//! Call tracer formatter implementation.

use evm_tracing_events::MarshalledOpcode;
use sp_core::sp_std::cmp::Ordering;

use crate::{
    listeners::call_list::Listener,
    types::{
        block::BlockTransactionTrace,
        blockscout::BlockscoutCallInner,
        call_tracer::{CallTracerCall, CallTracerInner},
        single::{Call, TransactionTrace},
        CallType, CreateResult,
    },
};

/// Call tracer formatter.
pub struct Formatter;

impl super::ResponseFormatter for Formatter {
    type Listener = Listener;
    type Response = Vec<BlockTransactionTrace>;

    fn format(listener: Listener) -> Option<Vec<BlockTransactionTrace>> {
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
            let mut result: Vec<Call> = entry
                .iter()
                .map(|(_, it)| {
                    let from = it.from;
                    let trace_address = it.trace_address.clone();
                    let value = it.value;
                    let gas = it.gas;
                    let gas_used = it.gas_used;
                    let inner = it.inner.clone();
                    Call::CallTracer(CallTracerCall {
                        from,
                        gas,
                        gas_used,
                        trace_address: Some(trace_address.clone()),
                        inner: match inner.clone() {
                            BlockscoutCallInner::Call {
                                input,
                                to,
                                res,
                                call_type,
                            } => CallTracerInner::Call {
                                call_type: match call_type {
                                    CallType::Call => MarshalledOpcode::from("CALL"),
                                    CallType::CallCode => MarshalledOpcode::from("CALLCODE"),
                                    CallType::DelegateCall => {
                                        MarshalledOpcode::from("DELEGATECALL")
                                    }
                                    CallType::StaticCall => MarshalledOpcode::from("STATICCALL"),
                                },
                                to,
                                input,
                                res: res.clone(),
                                value: Some(value),
                            },
                            BlockscoutCallInner::Create { init, res } => CallTracerInner::Create {
                                input: init,
                                error: match res {
                                    CreateResult::Success { .. } => None,
                                    CreateResult::Error { ref error } => Some(error.clone()),
                                },
                                to: match res {
                                    CreateResult::Success {
                                        created_contract_address_hash,
                                        ..
                                    } => Some(created_contract_address_hash),
                                    CreateResult::Error { .. } => None,
                                },
                                output: match res {
                                    CreateResult::Success {
                                        created_contract_code,
                                        ..
                                    } => Some(created_contract_code),
                                    CreateResult::Error { .. } => None,
                                },
                                value,
                                call_type: MarshalledOpcode::from("CREATE"),
                            },
                            BlockscoutCallInner::SelfDestruct { balance, to } => {
                                CallTracerInner::SelfDestruct {
                                    value: balance,
                                    to,
                                    call_type: MarshalledOpcode::from("SELFDESTRUCT"),
                                }
                            }
                        },
                        calls: Vec::new(),
                    })
                })
                .collect();
            // Geth's `callTracer` expects a tree of nested calls and we have a stack.
            //
            // We iterate over the sorted stack, and push each children to it's
            // parent (the item which's `trace_address` matches &T[0..T.len()-1]) until there
            // is a single item on the list.
            //
            // The last remaining item is the context call with all it's descendants. I.e.
            //
            // 		# Input
            // 		[]
            // 		[0]
            // 		[0,0]
            // 		[0,0,0]
            // 		[0,1]
            // 		[0,1,0]
            // 		[0,1,1]
            // 		[0,1,2]
            // 		[1]
            // 		[1,0]
            //
            // 		# Sorted
            // 		[0,0,0] -> pop 0 and push to [0,0]
            // 		[0,1,0] -> pop 0 and push to [0,1]
            // 		[0,1,1] -> pop 1 and push to [0,1]
            // 		[0,1,2] -> pop 2 and push to [0,1]
            // 		[0,0] -> pop 0 and push to [0]
            // 		[0,1] -> pop 1 and push to [0]
            // 		[1,0] -> pop 0 and push to [1]
            // 		[0] -> pop 0 and push to root
            // 		[1] -> pop 1 and push to root
            // 		[]
            //
            // 		# Result
            // 		root {
            // 			calls: {
            // 				0 { 0 { 0 }, 1 { 0, 1, 2 }},
            // 				1 { 0 },
            // 			}
            // 		}
            if result.len() > 1 {
                // Sort the stack. Assume there is no `Ordering::Equal`, as we are
                // sorting by index.
                //
                // We consider an item to be `Ordering::Less` when:
                // 	- Is closer to the root or
                //	- Is greater than its sibling.
                result.sort_by(|a, b| match (a, b) {
                    (
                        Call::CallTracer(CallTracerCall {
                            trace_address: Some(a),
                            ..
                        }),
                        Call::CallTracer(CallTracerCall {
                            trace_address: Some(b),
                            ..
                        }),
                    ) => {
                        let a_len = a.len();
                        let b_len = b.len();
                        let sibling_greater_than = |a: &Vec<u32>, b: &Vec<u32>| -> bool {
                            for (i, a_value) in a.iter().enumerate() {
                                match a_value.cmp(&b[i]) {
                                    Ordering::Greater => return true,
                                    Ordering::Less => return false,
                                    Ordering::Equal => continue,
                                }
                            }

                            false
                        };
                        if b_len > a_len || (a_len == b_len && sibling_greater_than(a, b)) {
                            Ordering::Less
                        } else {
                            Ordering::Greater
                        }
                    }
                    _ => unreachable!(),
                });
                // Stack pop-and-push.
                while result.len() > 1 {
                    let mut last = result
                        .pop()
                        .expect("result.len() > 1, so pop() necessarily returns an element");
                    // Find the parent index.
                    if let Some(index) =
                        result
                            .iter()
                            .position(|current| match (last.clone(), current) {
                                (
                                    Call::CallTracer(CallTracerCall {
                                        trace_address: Some(a),
                                        ..
                                    }),
                                    Call::CallTracer(CallTracerCall {
                                        trace_address: Some(b),
                                        ..
                                    }),
                                ) => {
                                    &b[..]
                                        == a.get(
                                            0..a.len().checked_sub(1).expect(
                                                "valid operation due to the check before; qed.",
                                            ),
                                        )
                                        .expect("non-root element while traversing trace result")
                                }
                                _ => unreachable!(),
                            })
                    {
                        // Remove `trace_address` from result.
                        if let Call::CallTracer(CallTracerCall {
                            ref mut trace_address,
                            ..
                        }) = last
                        {
                            *trace_address = None;
                        }
                        // Push the children to parent.
                        if let Some(Call::CallTracer(CallTracerCall { calls, .. })) =
                            result.get_mut(index)
                        {
                            calls.push(last);
                        }
                    }
                }
            }
            // Remove `trace_address` from result.
            if let Some(Call::CallTracer(CallTracerCall { trace_address, .. })) = result.get_mut(0)
            {
                *trace_address = None;
            }
            if result.len() == 1 {
                traces.push(BlockTransactionTrace {
                    // u32 (eth tx index) is big enough for this truncation to be practically impossible.
                    tx_position: u32::try_from(eth_tx_index).unwrap(),
                    // Use default, the correct value will be set upstream
                    tx_hash: Default::default(),
                    result: TransactionTrace::CallListNested(
                        result
                            .pop()
                            .expect("result.len() == 1, so pop() necessarily returns this element"),
                    ),
                });
            }
        }
        if traces.is_empty() {
            return None;
        }

        Some(traces)
    }
}
