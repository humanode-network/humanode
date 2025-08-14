//! Raw formatter implementation.

use crate::{listeners::raw::Listener, types::single::TransactionTrace};

/// Raw formatter.
pub struct Formatter;

impl super::ResponseFormatter for Formatter {
    type Listener = Listener;
    type Response = TransactionTrace;

    fn format(listener: Listener) -> Option<TransactionTrace> {
        if listener.remaining_memory_usage.is_none() {
            None
        } else {
            Some(TransactionTrace::Raw {
                struct_logs: listener.struct_logs,
                gas: listener.final_gas.into(),
                return_value: listener.return_value,
            })
        }
    }
}
