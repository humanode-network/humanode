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
