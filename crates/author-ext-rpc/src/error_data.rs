//! The shapes and variants of the custom error data.

use serde::Serialize;

/// The RPC error context we provide to catch transaction pool errors.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorExtTxErrorDetails {
    /// The error kind.
    pub kind: AuthorExtTxErrorKind,
    /// The human-firendly message for what happened.
    pub message: &'static str,
    /// The message from the inner transaction pool error.
    pub inner_error: String,
}

/// The error kinds that we expose in the RPC that originate from the transaction pool.
#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthorExtTxErrorKind {
    /// The inability to pay some fees (e.g. account balance too low).
    NoFunds,
}
