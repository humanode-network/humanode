//! The validator related error.

/// Custom rpc error codes.
pub mod api_error_code {
    /// Validator key is not available.
    pub const MISSING_VALIDATOR_KEY: i32 = 500;

    /// Validator key extraction has failed.
    pub const VALIDATOR_KEY_EXTRACTION: i32 = 600;
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
