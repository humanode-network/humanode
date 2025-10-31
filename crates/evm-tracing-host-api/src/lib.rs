//! Environmental-aware externalities for EVM tracing in Wasm runtime. This enables
//! capturing the - potentially large - trace output data in the host and keep
//! a low memory footprint in `--execution=wasm`.
//!
//! - The original trace Runtime Api call is wrapped `using` environmental (thread local).
//! - Arguments are scale-encoded known types in the host.
//! - Host functions will decode the input and emit an event `with` environmental.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode;
use evm_tracing_events::{Event, EvmEvent, GasometerEvent, RuntimeEvent, StepEventFilter};
use sp_runtime_interface::runtime_interface;
use sp_std::vec::Vec;

/// EVM tracing runtime interface.
#[runtime_interface]
pub trait Externalities {
    /// An `EvmEvent` proxied by the runtime to this host function.
    /// EVM -> runtime -> host.
    fn evm_event(&mut self, event: Vec<u8>) {
        if let Ok(event) = EvmEvent::decode(&mut &event[..]) {
            Event::Evm(event).emit();
        }
    }

    /// A `GasometerEvent` proxied by the runtime to this host function.
    /// EVM gasometer -> runtime -> host.
    fn gasometer_event(&mut self, event: Vec<u8>) {
        if let Ok(event) = GasometerEvent::decode(&mut &event[..]) {
            Event::Gasometer(event).emit();
        }
    }

    /// A `RuntimeEvent` proxied by the runtime to this host function.
    /// EVM runtime -> runtime -> host.
    fn runtime_event(&mut self, event: Vec<u8>) {
        if let Ok(event) = RuntimeEvent::decode(&mut &event[..]) {
            Event::Runtime(event).emit();
        }
    }

    /// Allow the tracing module in the runtime to know how to filter Step event
    /// content, as cloning the entire data is expensive and most of the time
    /// not necessary.
    fn step_event_filter(&self) -> StepEventFilter {
        evm_tracing_events::step_event_filter().unwrap_or_default()
    }

    /// An event to create a new `CallList` (currently a new transaction when tracing a block).
    fn call_list_new(&mut self) {
        Event::CallListNew().emit();
    }
}
