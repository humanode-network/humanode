//! The validator related error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde_json::Value;

use super::ApiErrorCode;

/// The validator related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum ValidatorKeyError {
    /// Validator key not available.
    #[error("validator key not available")]
    MissingValidatorKey,
    /// Unable to extract validator key.
    #[error("unable to extract validator key")]
    ValidatorKeyExtraction,
}

impl From<ValidatorKeyError> for JsonRpseeError {
    fn from(err: ValidatorKeyError) -> Self {
        let (code, data): (ApiErrorCode, Option<Value>) = match err {
            ValidatorKeyError::MissingValidatorKey => (
                ApiErrorCode::MissingValidatorKey,
                Some(serde_json::json!({ "validator key not available": true })),
            ),
            ValidatorKeyError::ValidatorKeyExtraction => {
                (ApiErrorCode::ValidatorKeyExtraction, None)
            }
        };

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(code as _).code(),
            err.to_string(),
            data,
        )))
    }
}
