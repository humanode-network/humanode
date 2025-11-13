//! Raw listener.

use evm_tracing_events::{
    runtime::Capture, runtime::ExitReason, Event, GasometerEvent, Listener as ListenerT,
    MarshalledOpcode, RuntimeEvent, StepEventFilter,
};
use sp_core::{sp_std::collections::btree_map::BTreeMap, H160, H256};

use crate::types::{convert_memory, single::RawStepLog, ContextType};

/// Listener.
#[derive(Debug)]
pub struct Listener {
    /// Disable storage flag.
    disable_storage: bool,
    /// Disable memory flag.
    disable_memory: bool,
    /// Disable stack flag.
    disable_stack: bool,
    /// New context flag.
    new_context: bool,
    /// Context stack.
    context_stack: Vec<Context>,
    /// Logs.
    pub struct_logs: Vec<RawStepLog>,
    /// Return value.
    pub return_value: Vec<u8>,
    /// Final gas.
    pub final_gas: u64,
    /// Remaining memory usage.
    pub remaining_memory_usage: Option<usize>,
}

/// Context
#[derive(Debug)]
struct Context {
    /// Storage cache.
    storage_cache: BTreeMap<H256, H256>,
    /// Address.
    address: H160,
    /// Current step.
    current_step: Option<Step>,
    /// Global storage changes.
    global_storage_changes: BTreeMap<H160, BTreeMap<H256, H256>>,
}

/// Step.
#[derive(Debug)]
struct Step {
    /// Current opcode.
    opcode: MarshalledOpcode,
    /// Depth of the context.
    depth: usize,
    /// Remaining gas.
    gas: u64,
    /// Gas cost of the following opcode.
    gas_cost: u64,
    /// Program counter position.
    position: usize,
    /// EVM memory copy (if not disabled).
    memory: Option<Vec<u8>>,
    /// EVM stack copy (if not disabled).
    stack: Option<Vec<H256>>,
}

impl Listener {
    /// New listener.
    pub fn new(
        disable_storage: bool,
        disable_memory: bool,
        disable_stack: bool,
        raw_max_memory_usage: usize,
    ) -> Self {
        Self {
            disable_storage,
            disable_memory,
            disable_stack,
            remaining_memory_usage: Some(raw_max_memory_usage),
            struct_logs: vec![],
            return_value: vec![],
            final_gas: 0,
            new_context: false,
            context_stack: vec![],
        }
    }

    /// Run closure.
    pub fn using<R, F: FnOnce() -> R>(&mut self, f: F) -> R {
        evm_tracing_events::using(self, f)
    }

    /// Gasometer event.
    pub fn gasometer_event(&mut self, event: GasometerEvent) {
        match event {
            GasometerEvent::RecordTransaction { cost, .. } => {
                // First event of a transaction.
                // Next step will be the first context.
                self.new_context = true;
                self.final_gas = cost;
            }
            GasometerEvent::RecordCost { cost, snapshot } => {
                if let Some(context) = self.context_stack.last_mut() {
                    // Register opcode cost. (ignore costs not between Step and StepResult)
                    if let Some(step) = &mut context.current_step {
                        step.gas = snapshot.gas();
                        step.gas_cost = cost;
                    }

                    self.final_gas = snapshot.used_gas;
                }
            }
            GasometerEvent::RecordDynamicCost {
                gas_cost, snapshot, ..
            } => {
                if let Some(context) = self.context_stack.last_mut() {
                    // Register opcode cost. (ignore costs not between Step and StepResult)
                    if let Some(step) = &mut context.current_step {
                        step.gas = snapshot.gas();
                        step.gas_cost = gas_cost;
                    }

                    self.final_gas = snapshot.used_gas;
                }
            }
            // We ignore other kinds of message if any (new ones may be added in the future).
            #[allow(unreachable_patterns)]
            _ => (),
        }
    }

    /// Runtime event.
    pub fn runtime_event(&mut self, event: RuntimeEvent) {
        match event {
            RuntimeEvent::Step {
                context,
                opcode,
                position,
                stack,
                memory,
            } => {
                // Create a context if needed.
                if self.new_context {
                    self.new_context = false;

                    self.context_stack.push(Context {
                        storage_cache: BTreeMap::new(),
                        address: context.address,
                        current_step: None,
                        global_storage_changes: BTreeMap::new(),
                    });
                }

                let depth = self.context_stack.len();

                // Ignore steps outside of any context (shouldn't even be possible).
                if let Some(context) = self.context_stack.last_mut() {
                    context.current_step = Some(Step {
                        opcode,
                        depth,
                        gas: 0,      // 0 for now, will add with gas events
                        gas_cost: 0, // 0 for now, will add with gas events
                        // usize (position) is big enough for this truncation to be practically impossible.
                        position: usize::try_from(*position.as_ref().unwrap_or(&0)).unwrap(),
                        memory: if self.disable_memory {
                            None
                        } else {
                            let memory = memory.expect("memory data to not be filtered out");

                            self.remaining_memory_usage = self
                                .remaining_memory_usage
                                .and_then(|inner| inner.checked_sub(memory.data.len()));

                            if self.remaining_memory_usage.is_none() {
                                return;
                            }

                            Some(memory.data.clone())
                        },
                        stack: if self.disable_stack {
                            None
                        } else {
                            let stack = stack.expect("stack data to not be filtered out");

                            self.remaining_memory_usage = self
                                .remaining_memory_usage
                                .and_then(|inner| inner.checked_sub(stack.data.len()));

                            if self.remaining_memory_usage.is_none() {
                                return;
                            }

                            Some(stack.data.clone())
                        },
                    });
                }
            }
            RuntimeEvent::StepResult {
                result,
                return_value,
            } => {
                // StepResult is expected to be emitted after a step (in a context).
                // Only case StepResult will occur without a Step before is in a transfer
                // transaction to a non-contract address. However it will not contain any
                // steps and return an empty trace, so we can ignore this edge case.
                if let Some(context) = self.context_stack.last_mut() {
                    if let Some(current_step) = context.current_step.take() {
                        let Step {
                            opcode,
                            depth,
                            gas,
                            gas_cost,
                            position,
                            memory,
                            stack,
                        } = current_step;

                        let memory = memory.map(convert_memory);

                        let storage = if self.disable_storage {
                            None
                        } else {
                            self.remaining_memory_usage =
                                self.remaining_memory_usage.and_then(|inner| {
                                    inner
                                        .checked_sub(context.storage_cache.len().saturating_mul(64))
                                });

                            if self.remaining_memory_usage.is_none() {
                                return;
                            }

                            Some(context.storage_cache.clone())
                        };

                        self.struct_logs.push(RawStepLog {
                            depth: depth.into(),
                            gas: gas.into(),
                            gas_cost: gas_cost.into(),
                            memory,
                            op: opcode,
                            pc: position.into(),
                            stack,
                            storage,
                        });
                    }
                }

                // We match on the capture to handle traps/exits.
                match result {
                    Err(Capture::Exit(reason)) => {
                        // Exit = we exit the context (should always be some)
                        if let Some(mut context) = self.context_stack.pop() {
                            // If final context is exited, we store gas and return value.
                            if self.context_stack.is_empty() {
                                self.return_value = return_value.to_vec();
                            }

                            // If the context exited without revert we must keep track of the
                            // updated storage keys.
                            if !self.disable_storage && matches!(reason, ExitReason::Succeed(_)) {
                                if let Some(parent_context) = self.context_stack.last_mut() {
                                    // Add cache to storage changes.
                                    context
                                        .global_storage_changes
                                        .insert(context.address, context.storage_cache);

                                    // Apply storage changes to parent, either updating its cache or map of changes.
                                    for (address, mut storage) in context.global_storage_changes {
                                        // Same address => We update its cache (only tracked keys)
                                        if parent_context.address == address {
                                            for (cached_key, cached_value) in
                                                &mut parent_context.storage_cache
                                            {
                                                if let Some(value) = storage.remove(cached_key) {
                                                    *cached_value = value;
                                                }
                                            }
                                        }
                                        // Otherwise, update the storage changes.
                                        else {
                                            parent_context
                                                .global_storage_changes
                                                .entry(address)
                                                .or_insert_with(BTreeMap::new)
                                                .append(&mut storage);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(Capture::Trap(opcode)) if ContextType::from(opcode.clone()).is_some() => {
                        self.new_context = true;
                    }
                    _ => (),
                }
            }
            RuntimeEvent::SLoad {
                address: _,
                index,
                value,
            }
            | RuntimeEvent::SStore {
                address: _,
                index,
                value,
            } => {
                if let Some(context) = self.context_stack.last_mut() {
                    if !self.disable_storage {
                        context.storage_cache.insert(index, value);
                    }
                }
            }
            // We ignore other kinds of messages if any (new ones may be added in the future).
            #[allow(unreachable_patterns)]
            _ => (),
        }
    }
}

impl ListenerT for Listener {
    fn event(&mut self, event: Event) {
        if self.remaining_memory_usage.is_none() {
            return;
        }

        match event {
            Event::Gasometer(e) => self.gasometer_event(e),
            Event::Runtime(e) => self.runtime_event(e),
            _ => {}
        };
    }

    fn step_event_filter(&self) -> StepEventFilter {
        StepEventFilter {
            enable_memory: !self.disable_memory,
            enable_stack: !self.disable_stack,
        }
    }
}
