//! The `enroll` method error kinds.

use jsonrpsee::core::Error as JsonRpseeError;
use rpc_validator_key_logic::ValidatorKeyError;

use super::{robonode::RobonodeError, sign::SignError};

/// The `enroll` method error kinds.
#[derive(Debug)]
pub enum EnrollError {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during doing a call into robonode.
    Robonode(RobonodeError),
    /// An error that can occur during signing process.
    Sign(SignError),
}

impl From<EnrollError> for JsonRpseeError {
    fn from(err: EnrollError) -> Self {
        match err {
            EnrollError::KeyExtraction(err) => err.into(),
            EnrollError::Robonode(err) => err.into(),
            EnrollError::Sign(err) => err.into(),
        }
    }
}
