//! The `enroll` method error.

use rpc_validator_key_logic::Error as ValidatorKeyError;

use super::{api_error_code, sign::Error as SignError};
use crate::error_data;

/// The `enroll` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during doing a call into robonode.
    Robonode(robonode_client::Error<robonode_client::EnrollError>),
    /// An error that can occur during signing process.
    Sign(SignError),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::KeyExtraction(err) => {
                rpc_error_response::simple(api_error_code::RUNTIME_API, err.to_string())
            }
            Error::Robonode(
                err @ robonode_client::Error::Call(robonode_client::EnrollError::FaceScanRejected),
            ) => rpc_error_response::data(
                api_error_code::ROBONODE,
                err.to_string(),
                error_data::ShouldRetry,
            ),
            Error::Robonode(err) => {
                rpc_error_response::simple(api_error_code::ROBONODE, err.to_string())
            }
            Error::Sign(err) => rpc_error_response::simple(api_error_code::SIGN, err.to_string()),
        }
    }
}
