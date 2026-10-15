//! The shapes and variants of the custom error data.

use serde::Serialize;

use crate::step_context::StepContext;

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

/// The RPC error with scan result blob data.
pub struct ScanResultBlob(pub String);

impl Serialize for ScanResultBlob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_json::json!({ "scanResultBlob": self.0 }).serialize(serializer)
    }
}

/// The RPC error that would either provide a scan result blob, or specify that a retry is in order.
///
/// The case where no scan result blob is present, but a retry is communicated is possible when
/// the new flow is used with an old robonode that doesn't return scan result blobs yet.
#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum BlobOrRetry {
    /// The scan result blob data.
    ScanResultBlob(ScanResultBlob),

    /// The should retry data.
    ShouldRetry(ShouldRetry),
}

impl From<ShouldRetry> for BlobOrRetry {
    fn from(value: ShouldRetry) -> Self {
        Self::ShouldRetry(value)
    }
}

impl From<ScanResultBlob> for BlobOrRetry {
    fn from(value: ScanResultBlob) -> Self {
        Self::ScanResultBlob(value)
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

/// The RPC error context for a step that failed with the chain still open: the web app passes
/// the scan result blob to the FaceTec SDK and retries the step with the new context.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepRetry {
    /// The response blob to pass to the FaceTec SDK.
    pub scan_result_blob: String,

    /// The opaque context to pass to the next step.
    pub context: sp_core::Bytes,
}

impl StepRetry {
    /// Builds the retry data from the error chain, if the chain is still open.
    pub fn from_error_chain(chain: &biometrics_api_client::ErrorChain) -> Option<Self> {
        match chain {
            biometrics_api_client::ErrorChain::Closed => None,
            biometrics_api_client::ErrorChain::Open {
                response_blob,
                chain,
            } => Some(Self {
                scan_result_blob: response_blob.clone(),
                context: StepContext::from(chain).to_bytes(),
            }),
        }
    }
}
