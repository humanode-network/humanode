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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn expected_should_retry_error() {
        let error: JsonRpseeError = RobonodeError::ShouldRetry.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":200,\"message\":\"request to the robonode failed\",\"data\":{\"shouldRetry\":true}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_other_error() {
        let error: JsonRpseeError = RobonodeError::Other("test".to_string()).into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"request to the robonode failed with custom error: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
