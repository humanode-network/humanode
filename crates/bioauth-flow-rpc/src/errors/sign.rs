//! The signer related error kinds.

/// The signer related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum SignError {
    /// Signing process failed.
    #[error("signing failed")]
    SigningFailed,
}
