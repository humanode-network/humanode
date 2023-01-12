//! The runtime api error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

/// The runtime api error kinds.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeApiError {
    /// The runtime native error.
    #[error("runtime error: {0}")]
    Native(String),
    /// The error during session keys decoding.
    #[error("error during session keys decoding: {0}")]
    SessionKeysDecoding(String),
    /// The error during the creation of the signed set keys extrinsic.
    #[error("error during the creation of the signed set keys extrinsic")]
    CreatingSignedSetKeys,
}

impl From<RuntimeApiError> for JsonRpseeError {
    fn from(err: RuntimeApiError) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
            err.to_string(),
            None::<()>,
        )))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn expected_native_error() {
        let error: JsonRpseeError = RuntimeApiError::Native("test".to_string()).into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":300,\"message\":\"runtime error: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_session_key_decoding_error() {
        let error: JsonRpseeError = RuntimeApiError::SessionKeysDecoding("test".to_string()).into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":300,\"message\":\"error during session keys decoding: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_creating_signed_set_keys_error() {
        let error: JsonRpseeError = RuntimeApiError::CreatingSignedSetKeys.into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":300,\"message\":\"error during the creation of the signed set keys extrinsic\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
