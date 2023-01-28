//! The `enroll` method error.

use rpc_validator_key_logic::ValidatorKeyError;

use super::{app, sign::SignError, ApiErrorCode};
use crate::error_data;

/// The `enroll` method error kinds.
#[derive(Debug)]
pub enum EnrollError {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during doing a call into robonode.
    Robonode(robonode_client::Error<robonode_client::EnrollError>),
    /// An error that can occur during signing process.
    Sign(SignError),
}

impl From<EnrollError> for jsonrpsee::core::Error {
    fn from(err: EnrollError) -> Self {
        match err {
            EnrollError::KeyExtraction(err) => {
                app::simple(ApiErrorCode::RuntimeApi, err.to_string())
            }
            EnrollError::Robonode(
                err @ robonode_client::Error::Call(robonode_client::EnrollError::FaceScanRejected),
            ) => app::data(
                ApiErrorCode::Robonode,
                err.to_string(),
                error_data::ShouldRetry,
            ),
            EnrollError::Robonode(err) => app::simple(ApiErrorCode::Robonode, err.to_string()),
            EnrollError::Sign(err) => app::simple(ApiErrorCode::Sign, err.to_string()),
        }
    }
}
