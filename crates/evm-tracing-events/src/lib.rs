//! EVM tracing events related primitives.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_core::{H160, U256};
use sp_runtime_interface::pass_by::PassByCodec;

pub mod evm;
pub mod gasometer;
pub mod runtime;

pub use gasometer::GasometerEvent;
pub use runtime::RuntimeEvent;

pub use self::evm::EvmEvent;

environmental::environmental!(listener: dyn Listener + 'static);

/// Run closure with provided listener.
pub fn using<R, F: FnOnce() -> R>(l: &mut (dyn Listener + 'static), f: F) -> R {
    listener::using(l, f)
}

/// Allow to configure which data of the step event
/// we want to keep or discard. Not discarding the data requires cloning the data
/// in the runtime which have a significant cost for each step.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Default, PassByCodec)]
pub struct StepEventFilter {
    /// Enabling stack flag.
    pub enable_stack: bool,
    /// Enabling memory flag.
    pub enable_memory: bool,
}

/// EVM tracing events.
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub enum Event {
    /// EVM explicit event.
    Evm(evm::EvmEvent),
    /// EVM gasometer event.
    Gasometer(gasometer::GasometerEvent),
    /// EVM runtime event.
    Runtime(runtime::RuntimeEvent),
    /// An event used to create a new `CallList`.
    CallListNew(),
}

impl Event {
    /// Access the global reference and call it's `event` method, passing the `Event` itself as
    /// argument.
    ///
    /// This only works if we are `using` a global reference to a `Listener` implementor.
    pub fn emit(self) {
        listener::with(|listener| listener.event(self));
    }
}

/// Main trait to proxy emitted messages.
/// Used 2 times :
/// - Inside the runtime to proxy the events through the host functions
/// - Inside the client to forward those events to the client listener.
pub trait Listener {
    /// Proxy emitted event.
    fn event(&mut self, event: Event);

    /// Allow the runtime to know which data should be discarded and not cloned.
    /// WARNING: It is only called once when the runtime tracing is instantiated to avoid
    /// performing many ext calls.
    fn step_event_filter(&self) -> StepEventFilter;
}

/// Allow the tracing module in the runtime to know how to filter Step event
/// content, as cloning the entire data is expensive and most of the time
/// not necessary.
pub fn step_event_filter() -> Option<StepEventFilter> {
    let mut filter = None;
    listener::with(|listener| filter = Some(listener.step_event_filter()));
    filter
}

/// EVM context of the runtime.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct Context {
    /// Execution address.
    pub address: H160,
    /// Caller of the EVM.
    pub caller: H160,
    /// Apparent value of the EVM.
    pub apparent_value: U256,
}

impl From<evm_runtime::Context> for Context {
    fn from(context: evm_runtime::Context) -> Self {
        Self {
            address: context.address,
            caller: context.caller,
            apparent_value: context.apparent_value,
        }
    }
}
