//! The signer related error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

/// The signer related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum SignerError {
    /// Signing process failed.
    #[error("signing failed")]
    SigningFailed,
}

impl From<SignerError> for JsonRpseeError {
    fn from(err: SignerError) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::Signer as _).code(),
            err.to_string(),
            None::<()>,
        )))
    }
}
