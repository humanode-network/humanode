//! The `get_validator_public_key` method error kinds.

use jsonrpsee::core::Error as JsonRpseeError;
use rpc_validator_key_logic::ValidatorKeyError;

/// The `get_validator_public_key` method error kinds.
#[derive(Debug)]
pub enum GetValidatorPublicKeyError {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
}

impl From<GetValidatorPublicKeyError> for JsonRpseeError {
    fn from(err: GetValidatorPublicKeyError) -> Self {
        match err {
            GetValidatorPublicKeyError::KeyExtraction(err) => err.into(),
        }
    }
}
