//! Formatters implementation.

use primitives_evm_tracing_events::Listener;
use serde::Serialize;

pub mod blockscout;
pub mod raw;
pub mod trace_filter;

/// Response formatter.
pub trait ResponseFormatter {
    /// Listener type.
    type Listener: Listener;
    /// Response type.
    type Response: Serialize;

    /// Format.
    fn format(listener: Self::Listener) -> Option<Self::Response>;
}
