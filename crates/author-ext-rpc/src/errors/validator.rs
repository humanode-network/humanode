//! The validator related error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde_json::Value;

use super::ApiErrorCode;

/// The validator related error kinds.
#[derive(Debug, thiserror::Error)]
pub enum ValidatorKeyError {
    /// Validator key not available.
    #[error("validator key not available")]
    MissingValidatorKey,
    /// Unable to extract own key.
    #[error("unable to extract own key")]
    ValidatorKeyExtraction,
}

impl From<ValidatorKeyError> for JsonRpseeError {
    fn from(err: ValidatorKeyError) -> Self {
        let (code, data): (ApiErrorCode, Option<Value>) = match err {
            ValidatorKeyError::MissingValidatorKey => (
                ApiErrorCode::MissingValidatorKey,
                Some(serde_json::json!({ "validatorKeyNotAvailable": true })),
            ),
            ValidatorKeyError::ValidatorKeyExtraction => {
                (ApiErrorCode::ValidatorKeyExtraction, None)
            }
        };

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(code as _).code(),
            err.to_string(),
            data,
        )))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn expected_validator_key_extraction_error() {
        let error: JsonRpseeError = ValidatorKeyError::ValidatorKeyExtraction.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":600,\"message\":\"unable to extract own key\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_missing_validator_key_error() {
        let error: JsonRpseeError = ValidatorKeyError::MissingValidatorKey.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":500,\"message\":\"validator key not available\",\"data\":{\"validatorKeyNotAvailable\":true}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
