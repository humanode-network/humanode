//! The `status` method error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use rpc_validator_key_logic::ValidatorKeyError;
use sp_api::ApiError;

use super::ApiErrorCode;

/// The `status` method error kinds.
#[derive(Debug)]
pub enum StatusError {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during doing a call into runtime api.
    RuntimeApi(ApiError),
}

impl From<StatusError> for JsonRpseeError {
    fn from(err: StatusError) -> Self {
        match err {
            StatusError::KeyExtraction(err) => err.into(),
            StatusError::RuntimeApi(err) => {
                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
                    format!("unable to get status from the runtime: {err}"),
                    None::<()>,
                )))
            }
        }
    }
}
