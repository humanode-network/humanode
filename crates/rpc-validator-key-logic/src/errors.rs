//! The validator related error kinds.

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
pub enum ApiErrorCode {
    /// Validator key is not available.
    MissingValidatorKey = 500,
    /// Validator key extraction has failed.
    ValidatorKeyExtraction = 600,
}

/// The validator related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Validator key not available.
    #[error("validator key not available")]
    MissingValidatorKey,
    /// Unable to extract own key.
    #[error("unable to extract own key")]
    ValidatorKeyExtraction,
}
