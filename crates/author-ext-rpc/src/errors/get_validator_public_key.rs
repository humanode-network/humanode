//! The `get_validator_public_key` method error kinds.

use rpc_validator_key_logic::ValidatorKeyError;

use super::{app, ApiErrorCode};

/// The `get_validator_public_key` method error kinds.
#[derive(Debug)]
pub enum GetValidatorPublicKeyError {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
}

impl From<GetValidatorPublicKeyError> for jsonrpsee::core::Error {
    fn from(err: GetValidatorPublicKeyError) -> Self {
        match err {
            GetValidatorPublicKeyError::KeyExtraction(
                err @ ValidatorKeyError::MissingValidatorKey,
            ) => app::data(
                ApiErrorCode::MissingValidatorKey,
                err.to_string(),
                rpc_validator_key_logic::error_data::ValidatorKeyNotAvailable,
            ),
            GetValidatorPublicKeyError::KeyExtraction(
                err @ ValidatorKeyError::ValidatorKeyExtraction,
            ) => app::simple(ApiErrorCode::ValidatorKeyExtraction, err.to_string()),
        }
    }
}
