use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde_json::Value;

use crate::ApiErrorCode;

#[derive(Debug)]
pub enum RobonodeError {
    ShouldRetry(ShouldRetryDetails),
    Other,
}

impl From<RobonodeError> for JsonRpseeError {
    fn from(robonode_err: RobonodeError) -> Self {
        let (code, data): (ApiErrorCode, Option<Value>) = match robonode_err {
            RobonodeError::ShouldRetry(details) => (ApiErrorCode::Robonode, Some(details.into())),
            RobonodeError::Other => (ApiErrorCode::Robonode, None),
        };

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(code as _).code(),
            "Robonode Error",
            data,
        )))
    }
}

#[derive(Debug)]
pub struct ShouldRetryDetails;

impl From<ShouldRetryDetails> for Value {
    fn from(_: ShouldRetryDetails) -> Self {
        serde_json::json!({ "shouldRetry": true })
    }
}
