//! The `get_validator_public_key` method error kinds.

use rpc_validator_key_logic::Error as ValidatorKeyError;

use super::{app, ApiErrorCode};

/// The `get_validator_public_key` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::KeyExtraction(err @ ValidatorKeyError::MissingValidatorKey) => app::data(
                ApiErrorCode::MissingValidatorKey,
                err.to_string(),
                rpc_validator_key_logic::error_data::ValidatorKeyNotAvailable,
            ),
            Error::KeyExtraction(err @ ValidatorKeyError::ValidatorKeyExtraction) => {
                app::simple(ApiErrorCode::ValidatorKeyExtraction, err.to_string())
            }
        }
    }
}
