use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

#[derive(Debug)]
pub enum RuntimeApiError {
    BioauthStatus(String),
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
