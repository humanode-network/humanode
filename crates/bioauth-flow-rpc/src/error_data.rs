//! The shapes and variants of the custom error data.

use serde::Serialize;

/// The RPC error context we provide to trigger the face capture logic again,
/// effectively requesting a retry of the same request with a new liveness data.
#[derive(Debug)]
pub struct ShouldRetry;

impl Serialize for ShouldRetry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_json::json!({ "shouldRetry": true }).serialize(serializer)
    }
}

/// The RPC error context we provide to handle scan result blob data.
pub struct ScanResultBlob(pub String);

impl Serialize for ScanResultBlob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_json::json!({ "scanResultBlob": self.0 }).serialize(serializer)
    }
}

/// The RPC error context we provide to describe transaction pool errors.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BioauthTxErrorDetails {
    /// The error kind.
    pub kind: BioauthTxErrorKind,
    /// The human-friendly message for what happened.
    pub message: &'static str,
    /// The message from the inner transaction pool error.
    pub inner_error: String,
}

/// The error kinds that we expose in the RPC that originate from the transaction pool.
#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BioauthTxErrorKind {
    /// Auth ticket signature was not valid.
    AuthTicketSignatureInvalid,
    /// We were unable to parse the auth ticket (although its signature was supposed to be
    /// validated by now).
    UnableToParseAuthTicket,
    /// The nonce was already seen by the system.
    NonceAlreadyUsed,
    /// The aactive authentication issued by this ticket is still on.
    AlreadyAuthenticated,
}
