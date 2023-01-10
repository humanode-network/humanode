use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde::Serialize;
use serde_json::Value;

use crate::ApiErrorCode;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidatorKeyError {
    MissingValidatorKey,
    ValidatorKeyExtraction,
}

impl From<ValidatorKeyError> for JsonRpseeError {
    fn from(validator_err: ValidatorKeyError) -> Self {
        let (code, data): (ApiErrorCode, Option<Value>) = match validator_err {
            ValidatorKeyError::MissingValidatorKey => (
                ApiErrorCode::MissingValidatorKey,
                Some(serde_json::json!({ "validator key not available": true })),
            ),
            ValidatorKeyError::ValidatorKeyExtraction => {
                (ApiErrorCode::ValidatorKeyExtraction, None)
            }
        };

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(code as _).code(),
            "Validator Error",
            data,
        )))
    }
}
