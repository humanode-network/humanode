//! Blockscout formatter implementation.

use crate::{
    listeners::call_list::Listener,
    types::single::{Call, TransactionTrace},
};

/// Blockscout formatter.
pub struct Formatter;

impl super::ResponseFormatter for Formatter {
    type Listener = Listener;
    type Response = TransactionTrace;

    fn format(listener: Listener) -> Option<TransactionTrace> {
        if let Some(entry) = listener.entries.last() {
            return Some(TransactionTrace::CallList(
                entry
                    .iter()
                    .map(|(_, value)| Call::Blockscout(value.clone()))
                    .collect(),
            ));
        }
        None
    }
}
