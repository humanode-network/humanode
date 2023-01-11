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
    /// The runtime itself error.
    #[error("runtime error: {0}")]
    Runtime(String),
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
