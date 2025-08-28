//! Substrate EVM tracer.
//!
//! Enables tracing the EVM opcode execution and proxies EVM messages to the host functions.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use evm::tracing::{using as evm_using, EventListener as EvmListener};
use evm_gasometer::tracing::{using as gasometer_using, EventListener as GasometerListener};
use evm_runtime::tracing::{using as runtime_using, EventListener as RuntimeListener};
use evm_tracing_events::{EvmEvent, GasometerEvent, RuntimeEvent, StepEventFilter};
use sp_std::{cell::RefCell, rc::Rc};

/// Listener proxy.
struct ListenerProxy<T>(pub Rc<RefCell<T>>);

impl<T: GasometerListener> GasometerListener for ListenerProxy<T> {
    fn event(&mut self, event: evm_gasometer::tracing::Event) {
        self.0.borrow_mut().event(event);
    }
}

impl<T: RuntimeListener> RuntimeListener for ListenerProxy<T> {
    fn event(&mut self, event: evm_runtime::tracing::Event) {
        self.0.borrow_mut().event(event);
    }
}

impl<T: EvmListener> EvmListener for ListenerProxy<T> {
    fn event(&mut self, event: evm::tracing::Event) {
        self.0.borrow_mut().event(event);
    }
}

/// EVM tracer.
pub struct EvmTracer {
    /// Step event filter.
    step_event_filter: StepEventFilter,
}

impl Default for EvmTracer {
    fn default() -> Self {
        Self {
            step_event_filter: evm_tracing_host_api::externalities::step_event_filter(),
        }
    }
}

impl EvmTracer {
    /// Setup event listeners and execute provided closure.
    ///
    /// Consume the tracer and return it alongside the return value of
    /// the closure.
    pub fn trace<R, F: FnOnce() -> R>(self, f: F) {
        let wrapped = Rc::new(RefCell::new(self));

        let mut gasometer = ListenerProxy(Rc::clone(&wrapped));
        let mut runtime = ListenerProxy(Rc::clone(&wrapped));
        let mut evm = ListenerProxy(Rc::clone(&wrapped));

        // Each line wraps the previous `f` into a `using` call.
        // Listening to new events results in adding one new line.
        // Order is irrelevant when registering listeners.
        let f = || runtime_using(&mut runtime, f);
        let f = || gasometer_using(&mut gasometer, f);
        let f = || evm_using(&mut evm, f);
        f();
    }

    /// Emit new call stack.
    pub fn emit_new() {
        evm_tracing_host_api::externalities::call_list_new();
    }
}

impl EvmListener for EvmTracer {
    fn event(&mut self, event: evm::tracing::Event) {
        let event: EvmEvent = event.into();
        let message = event.encode();
        evm_tracing_host_api::externalities::evm_event(message);
    }
}

impl GasometerListener for EvmTracer {
    fn event(&mut self, event: evm_gasometer::tracing::Event) {
        let event: GasometerEvent = event.into();
        let message = event.encode();
        evm_tracing_host_api::externalities::gasometer_event(message);
    }
}

impl RuntimeListener for EvmTracer {
    fn event(&mut self, event: evm_runtime::tracing::Event) {
        let event = RuntimeEvent::from_evm_event(event, self.step_event_filter);
        let message = event.encode();
        evm_tracing_host_api::externalities::runtime_event(message);
    }
}
