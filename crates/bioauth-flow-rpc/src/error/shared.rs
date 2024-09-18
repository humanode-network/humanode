//! The robonode requests related error.

use rpc_validator_key_logic::Error as ValidatorKeyError;
use serde::Serialize;

use super::{code, sign::Error as SignError};

/// The robonode requests related error.
#[derive(Debug)]
pub enum FlowBaseError<T: std::error::Error + 'static> {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during signing process.
    Sign(SignError),
    /// An error that can occur during doing a call into robonode.
    RobonodeClient(robonode_client::Error<T>),
}

impl<T: std::error::Error + 'static> FlowBaseError<T> {
    /// Convert the flow base error to the raw response error object.
    pub fn to_jsonrpsee_error<R, D: Serialize>(
        &self,
        robonode_call_error_data: R,
    ) -> jsonrpsee::core::Error
    where
        R: for<'a> Fn(&'a T) -> Option<D>,
    {
        match self {
            Self::KeyExtraction(err @ ValidatorKeyError::MissingValidatorKey) => {
                rpc_error_response::data(
                    code::MISSING_VALIDATOR_KEY,
                    err.to_string(),
                    rpc_validator_key_logic::error_data::ValidatorKeyNotAvailable,
                )
            }
            Self::KeyExtraction(err @ ValidatorKeyError::ValidatorKeyExtraction) => {
                rpc_error_response::simple(code::VALIDATOR_KEY_EXTRACTION, err.to_string())
            }
            Self::Sign(err) => rpc_error_response::simple(code::SIGN, err.to_string()),
            Self::RobonodeClient(err @ robonode_client::Error::Call(inner)) => {
                let maybe_data = (robonode_call_error_data)(inner);
                rpc_error_response::raw(code::ROBONODE, err.to_string(), maybe_data)
            }
            Self::RobonodeClient(err @ robonode_client::Error::Reqwest(_)) => {
                rpc_error_response::simple(code::ROBONODE, err.to_string())
            }
        }
    }
}
