use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde_json::Value;

use crate::ApiErrorCode;

#[derive(Debug)]
pub enum ValidatorKeyError {
    MissingValidatorKey(ValidatorKeyNotAvailable),
    ValidatorKeyExtraction,
}

impl From<ValidatorKeyError> for JsonRpseeError {
    fn from(validator_err: ValidatorKeyError) -> Self {
        let (code, data): (ApiErrorCode, Option<Value>) = match validator_err {
            ValidatorKeyError::MissingValidatorKey(details) => {
                (ApiErrorCode::MissingValidatorKey, Some(details.into()))
            }
            ValidatorKeyError::ValidatorKeyExtraction => {
                (ApiErrorCode::ValidatorKeyExtraction, None)
            }
        };

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(code as _).code(),
            "Validator Key Error",
            data,
        )))
    }
}

#[derive(Debug)]
pub struct ValidatorKeyNotAvailable;

impl From<ValidatorKeyNotAvailable> for Value {
    fn from(_: ValidatorKeyNotAvailable) -> Self {
        serde_json::json!({ "validator key not available": true })
    }
}
