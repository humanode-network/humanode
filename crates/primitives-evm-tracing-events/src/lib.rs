//! EVM tracing events related primitives.

#![cfg_attr(not(feature = "std"), no_std)]
// TODO: fix clippy.
#![allow(missing_docs)]

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

pub fn using<R, F: FnOnce() -> R>(l: &mut (dyn Listener + 'static), f: F) -> R {
    listener::using(l, f)
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Default, PassByCodec)]
pub struct StepEventFilter {
    pub enable_stack: bool,
    pub enable_memory: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub enum Event {
    Evm(evm::EvmEvent),
    Gasometer(gasometer::GasometerEvent),
    Runtime(runtime::RuntimeEvent),
    CallListNew(),
}

impl Event {
    pub fn emit(self) {
        listener::with(|listener| listener.event(self));
    }
}

pub trait Listener {
    fn event(&mut self, event: Event);

    fn step_event_filter(&self) -> StepEventFilter;
}

pub fn step_event_filter() -> Option<StepEventFilter> {
    let mut filter = None;
    listener::with(|listener| filter = Some(listener.step_event_filter()));
    filter
}

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
