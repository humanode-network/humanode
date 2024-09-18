//! The `status` method error.

use rpc_validator_key_logic::Error as ValidatorKeyError;
use sp_api::ApiError;

use crate::error;

/// The `status` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during validator key extraction.
    /// Specifically the validator key extraction failure, not the missing key.
    ValidatorKeyExtraction,
    /// An error that can occur during doing a call into runtime api.
    RuntimeApi(ApiError),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::ValidatorKeyExtraction => rpc_error_response::simple(
                error::code::VALIDATOR_KEY_EXTRACTION,
                ValidatorKeyError::ValidatorKeyExtraction.to_string(),
            ),
            Error::RuntimeApi(err) => rpc_error_response::simple(
                error::code::RUNTIME_API,
                format!("unable to get status from the runtime: {err}"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {

    use jsonrpsee::types::ErrorObject;

    use super::*;

    #[test]
    fn error_validator_key_extraction() {
        let error: jsonrpsee::core::Error = Error::ValidatorKeyExtraction.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":600,\"message\":\"unable to extract own key\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_runtime_api() {
        let error: jsonrpsee::core::Error =
            Error::RuntimeApi(ApiError::Application("test".into())).into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":300,\"message\":\"unable to get status from the runtime: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
