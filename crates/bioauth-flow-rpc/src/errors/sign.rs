//! The signer related error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use rpc_validator_key_logic::ValidatorKeyError;

use super::ApiErrorCode;

/// The signer related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum SignError {
    /// Validator key extraction error.
    #[error("validator key: {0}")]
    ValidatorKey(ValidatorKeyError),
    /// Signing process failed.
    #[error("signing failed")]
    SigningFailed,
}

impl From<SignError> for JsonRpseeError {
    fn from(err: SignError) -> Self {
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
        let error: JsonRpseeError = SignError::SigningFailed.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":100,\"message\":\"signing failed\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
