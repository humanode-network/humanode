//! The `get_validator_public_key` method error kinds.

use rpc_validator_key_logic::Error as ValidatorKeyError;

use super::api_error_code;

/// The `get_validator_public_key` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::KeyExtraction(err @ ValidatorKeyError::MissingValidatorKey) => {
                rpc_error_response::data(
                    api_error_code::MISSING_VALIDATOR_KEY,
                    err.to_string(),
                    rpc_validator_key_logic::error_data::ValidatorKeyNotAvailable,
                )
            }
            Error::KeyExtraction(err @ ValidatorKeyError::ValidatorKeyExtraction) => {
                rpc_error_response::simple(
                    api_error_code::VALIDATOR_KEY_EXTRACTION,
                    err.to_string(),
                )
            }
        }
    }
}
