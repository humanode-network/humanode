//! The `authenticate_v2` method error.

use rpc_validator_key_logic::Error as ValidatorKeyError;

use super::{api_error_code, robonode_request::Error as RobonodeRequestError};
use crate::error_data::{self};

/// The `authenticate_v2` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during doing a request to robonode.
    RobonodeRequest(RobonodeRequestError<robonode_client::AuthenticateError>),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::RobonodeRequest(err) => match err {
                RobonodeRequestError::KeyExtraction(
                    err @ ValidatorKeyError::MissingValidatorKey,
                ) => rpc_error_response::data(
                    api_error_code::MISSING_VALIDATOR_KEY,
                    err.to_string(),
                    rpc_validator_key_logic::error_data::ValidatorKeyNotAvailable,
                ),
                RobonodeRequestError::KeyExtraction(
                    err @ ValidatorKeyError::ValidatorKeyExtraction,
                ) => rpc_error_response::simple(
                    api_error_code::VALIDATOR_KEY_EXTRACTION,
                    err.to_string(),
                ),
                RobonodeRequestError::Sign(err) => {
                    rpc_error_response::simple(api_error_code::SIGN, err.to_string())
                }
                RobonodeRequestError::Robonode(
                    ref err @ robonode_client::Error::Call(
                        robonode_client::AuthenticateError::PersonNotFoundReturnedBlob(
                            ref scan_result_blob,
                        )
                        | robonode_client::AuthenticateError::FaceScanRejectedReturnedBlob(
                            ref scan_result_blob,
                        )
                        | robonode_client::AuthenticateError::SignatureInvalidReturnedBlob(
                            ref scan_result_blob,
                        )
                        | robonode_client::AuthenticateError::LogicInternalReturnedBlob(
                            ref scan_result_blob,
                        ),
                    ),
                ) => rpc_error_response::data(
                    api_error_code::ROBONODE,
                    err.to_string(),
                    error_data::ScanResultBlob(scan_result_blob.clone()),
                ),
                RobonodeRequestError::Robonode(err) => {
                    rpc_error_response::simple(api_error_code::ROBONODE, err.to_string())
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use jsonrpsee::types::ErrorObject;

    use super::*;
    use crate::errors::sign::Error as SignError;

    #[test]
    fn error_key_extraction_validator_key_extraction() {
        let error: jsonrpsee::core::Error = Error::RobonodeRequest(
            RobonodeRequestError::KeyExtraction(ValidatorKeyError::ValidatorKeyExtraction),
        )
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":600,\"message\":\"unable to extract own key\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_key_extraction_missing_validator_key() {
        let error: jsonrpsee::core::Error = Error::RobonodeRequest(
            RobonodeRequestError::KeyExtraction(ValidatorKeyError::MissingValidatorKey),
        )
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":500,\"message\":\"validator key not available\",\"data\":{\"validatorKeyNotAvailable\":true}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_sign() {
        let error: jsonrpsee::core::Error =
            Error::RobonodeRequest(RobonodeRequestError::Sign(SignError::SigningFailed)).into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":100,\"message\":\"signing failed\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_face_scan_rejected() {
        let error: jsonrpsee::core::Error = Error::RobonodeRequest(RobonodeRequestError::Robonode(
            robonode_client::Error::Call(
                robonode_client::AuthenticateError::FaceScanRejectedReturnedBlob(
                    "scan result blob".to_owned(),
                ),
            ),
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: face scan rejected, returned blob\",\"data\":{\"scanResultBlob\":\"scan result blob\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_logic_internal() {
        let error: jsonrpsee::core::Error = Error::RobonodeRequest(RobonodeRequestError::Robonode(
            robonode_client::Error::Call(
                robonode_client::AuthenticateError::LogicInternalReturnedBlob(
                    "scan result blob".to_owned(),
                ),
            ),
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: logic internal error, returned blob\",\"data\":{\"scanResultBlob\":\"scan result blob\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_other() {
        let error: jsonrpsee::core::Error = Error::RobonodeRequest(RobonodeRequestError::Robonode(
            robonode_client::Error::Call(robonode_client::AuthenticateError::Unknown(
                "test".to_owned(),
            )),
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
}
