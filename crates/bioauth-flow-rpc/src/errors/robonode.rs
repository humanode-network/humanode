use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde_json::Value;

use super::ApiErrorCode;

#[derive(Debug)]
pub enum RobonodeError {
    ShouldRetry,
    Other(String),
}

impl From<RobonodeError> for JsonRpseeError {
    fn from(robonode_err: RobonodeError) -> Self {
        let code = ApiErrorCode::Robonode;

        let data: Option<Value> = match robonode_err {
            RobonodeError::ShouldRetry => Some(serde_json::json!({ "shouldRetry": true })),
            RobonodeError::Other(_) => None,
        };

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(code as _).code(),
            "Robonode Error",
            data,
        )))
    }
}
