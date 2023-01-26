//! The `set_keys` method error kinds.

use author_ext_api::CreateSignedSetKeysExtrinsicError;
use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use rpc_validator_key_logic::ValidatorKeyError;
use sp_api::ApiError;

use super::ApiErrorCode;

/// The `set_keys` method error kinds.
#[derive(Debug)]
pub enum SetKeysError {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during doing a call into runtime api.
    RuntimeAPi(ApiError),
    /// An error that can occur during signed `set_keys` extrinsic creation.
    ExtrinsicCreation(CreateSignedSetKeysExtrinsicError),
}

impl From<SetKeysError> for JsonRpseeError {
    fn from(err: SetKeysError) -> Self {
        let (code, message) = match err {
            SetKeysError::KeyExtraction(err) => return err.into(),
            SetKeysError::RuntimeAPi(err) => (ApiErrorCode::RuntimeApi, err.to_string()),
            SetKeysError::ExtrinsicCreation(err) => match err {
                CreateSignedSetKeysExtrinsicError::SessionKeysDecoding(err) => (
                    ApiErrorCode::RuntimeApi,
                    format!("Error during session keys decoding: {err}"),
                ),
                CreateSignedSetKeysExtrinsicError::SignedExtrinsicCreation => (
                    ApiErrorCode::RuntimeApi,
                    "Error during the creation of the signed set keys extrinsic".to_owned(),
                ),
            },
        };

        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(code as _).code(),
            message,
            None::<()>,
        )))
    }
}
