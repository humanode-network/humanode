//! The runtime api error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

/// The runtime api error kinds.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeApiError {
    /// Unable to get status from the runtime.
    #[error("unable to get status from the runtime: {0}")]
    BioauthStatus(String),
    /// Error creating authentication extrinsic.
    #[error("error creating authentication extrinsic: {0}")]
    CreatingAuthExtrinsic(String),
    /// The runtime native error.
    #[error("runtime error: {0}")]
    Native(String),
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
    fn expected_bioauth_status_error() {
        let error: JsonRpseeError = RuntimeApiError::BioauthStatus("test".to_string()).into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":300,\"message\":\"unable to get status from the runtime: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_creating_auth_extrinsic_error() {
        let error: JsonRpseeError =
            RuntimeApiError::CreatingAuthExtrinsic("test".to_string()).into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":300,\"message\":\"error creating authentication extrinsic: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

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
}
