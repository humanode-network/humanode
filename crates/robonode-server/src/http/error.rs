//! Error handle logic.

use serde::Serialize;

/// An API error serializable to JSON.
#[derive(Serialize)]
pub struct ErrorMessage {
    /// Status code rejection.
    pub code: u16,
    /// Message rejection.
    pub message: String,
}
