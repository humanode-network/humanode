//! The signer related error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

/// The signer related error kinds.
#[derive(Debug)]
pub enum SignerError {
    /// Signing process failed.
    SigningFailed,
}

impl From<SignerError> for JsonRpseeError {
    fn from(_: SignerError) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::Signer as _).code(),
            "Signer Error",
            None::<()>,
        )))
    }
}
