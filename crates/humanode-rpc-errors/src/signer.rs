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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn expected_signer_error() {
        let error: JsonRpseeError = SignerError::SigningFailed.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":100,\"message\":\"signing failed\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
