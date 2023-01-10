//! The runtime api error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

/// The runtime api error kinds.
#[derive(Debug)]
pub enum RuntimeApiError {
    /// Unable to get status from the runtime.
    BioauthStatus(String),
    /// Error creating authentication extrinsic.
    CreatingAuthExtrinsic(String),
}

impl From<RuntimeApiError> for JsonRpseeError {
    fn from(_: RuntimeApiError) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
            "Runtime Api Error",
            None::<()>,
        )))
    }
}
