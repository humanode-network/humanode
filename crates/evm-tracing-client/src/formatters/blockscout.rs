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

    fn format(mut listener: Listener) -> Option<TransactionTrace> {
        let entry = listener.entries.pop()?;

        Some(TransactionTrace::CallList(
            entry.into_values().map(Call::Blockscout).collect(),
        ))
    }
}
