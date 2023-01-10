//! The robonode related error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde_json::Value;

use super::ApiErrorCode;

/// The robonode related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum RobonodeError {
    /// The error to trigger the face capture logic again,
    /// effectively requesting a retry of the same request with a new liveness data.
    #[error("request to the robonode failed")]
    ShouldRetry,
    /// The request to the robonode failed with custom error.
    #[error("request to the robonode failed with custom error: {0}")]
    Other(String),
}

impl From<RobonodeError> for JsonRpseeError {
    fn from(err: RobonodeError) -> Self {
        let code = ApiErrorCode::Robonode;

        let data: Option<Value> = match err {
            RobonodeError::ShouldRetry => Some(serde_json::json!({ "shouldRetry": true })),
            RobonodeError::Other(_) => None,
        };

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(code as _).code(),
            err.to_string(),
            data,
        )))
    }
}
