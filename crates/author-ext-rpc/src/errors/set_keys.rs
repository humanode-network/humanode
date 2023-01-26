//! The `set_keys` method error kinds.

use author_ext_api::CreateSignedSetKeysExtrinsicError;
use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use sp_api::ApiError;

use super::ApiErrorCode;

/// The `set_keys` method error kinds.
#[derive(Debug)]
pub enum SetKeysError {
    /// An error that can occur during doing a call into runtime api.
    RuntimeAPi(ApiError),
    /// An error that can occur during signed `set_keys` extrinsic creation.
    ExtrinsicCreation(CreateSignedSetKeysExtrinsicError),
}

impl From<SetKeysError> for JsonRpseeError {
    fn from(err: SetKeysError) -> Self {
        let (code, message) = match err {
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

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn expected_runtime_api_error() {
//         let error: JsonRpseeError = RuntimeApiError::Native("test".to_string()).into();
//         let error: ErrorObject = error.into();

//         let expected_error_message = "{\"code\":300,\"message\":\"runtime error: test\"}";
//         assert_eq!(
//             expected_error_message,
//             serde_json::to_string(&error).unwrap()
//         );
//     }

//     #[test]
//     fn expected_session_key_decoding_error() {
//         let error: JsonRpseeError = RuntimeApiError::SessionKeysDecoding("test".to_string()).into();
//         let error: ErrorObject = error.into();

//         let expected_error_message =
//             "{\"code\":300,\"message\":\"error during session keys decoding: test\"}";
//         assert_eq!(
//             expected_error_message,
//             serde_json::to_string(&error).unwrap()
//         );
//     }

//     #[test]
//     fn expected_creating_signed_set_keys_error() {
//         let error: JsonRpseeError = RuntimeApiError::CreatingSignedSetKeys.into();
//         let error: ErrorObject = error.into();

//         let expected_error_message =
//             "{\"code\":300,\"message\":\"error during the creation of the signed set keys extrinsic\"}";
//         assert_eq!(
//             expected_error_message,
//             serde_json::to_string(&error).unwrap()
//         );
//     }
// }
