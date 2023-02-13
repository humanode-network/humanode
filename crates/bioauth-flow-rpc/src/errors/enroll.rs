//! The `enroll` method error.

use rpc_validator_key_logic::Error as ValidatorKeyError;

use super::{api_error_code, sign::Error as SignError};
use crate::error_data;

/// The `enroll` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during doing a call into robonode.
    Robonode(robonode_client::Error<robonode_client::EnrollError>),
    /// An error that can occur during signing process.
    Sign(SignError),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::KeyExtraction(err @ ValidatorKeyError::MissingValidatorKey) => {
                rpc_error_response::data(
                    api_error_code::MISSING_VALIDATOR_KEY,
                    err.to_string(),
                    rpc_validator_key_logic::error_data::ValidatorKeyNotAvailable,
                )
            }
            Error::KeyExtraction(err @ ValidatorKeyError::ValidatorKeyExtraction) => {
                rpc_error_response::simple(
                    api_error_code::VALIDATOR_KEY_EXTRACTION,
                    err.to_string(),
                )
            }
            Error::Robonode(
                err @ robonode_client::Error::Call(robonode_client::EnrollError::FaceScanRejected),
            ) => rpc_error_response::data(
                api_error_code::ROBONODE,
                err.to_string(),
                error_data::ShouldRetry,
            ),
            Error::Robonode(err) => {
                rpc_error_response::simple(api_error_code::ROBONODE, err.to_string())
            }
            Error::Sign(err) => rpc_error_response::simple(api_error_code::SIGN, err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {

    use jsonrpsee::types::ErrorObject;

    use super::*;

    #[test]
    fn error_key_extraction_validator_key_extraction() {
        let error: jsonrpsee::core::Error =
            Error::KeyExtraction(ValidatorKeyError::ValidatorKeyExtraction).into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":600,\"message\":\"unable to extract own key\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_key_extraction_missing_validator_key() {
        let error: jsonrpsee::core::Error =
            Error::KeyExtraction(ValidatorKeyError::MissingValidatorKey).into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":500,\"message\":\"validator key not available\",\"data\":{\"validatorKeyNotAvailable\":true}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_face_scan_rejected() {
        let error: jsonrpsee::core::Error = Error::Robonode(robonode_client::Error::Call(
            robonode_client::EnrollError::FaceScanRejected,
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: face scan rejected\",\"data\":{\"shouldRetry\":true}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_other() {
        let error: jsonrpsee::core::Error = Error::Robonode(robonode_client::Error::Call(
            robonode_client::EnrollError::Unknown("test".to_owned()),
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: unknown error: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_sign() {
        let error: jsonrpsee::core::Error = Error::Sign(SignError::SigningFailed).into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":100,\"message\":\"signing failed\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
