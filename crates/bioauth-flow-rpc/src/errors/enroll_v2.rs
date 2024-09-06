//! The `enroll_v2` method error.

use super::shared;
use crate::error_data;

/// The `enroll_v2` method error kinds.
#[derive(Debug)]
pub struct Error(pub shared::FlowBaseError<robonode_client::EnrollError>);

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        err.0
            .to_jsonrpsee_error::<_, error_data::BlobOrRetry>(|call_error| match call_error {
                robonode_client::EnrollError::FaceScanRejected(ref scan_result_blob)
                | robonode_client::EnrollError::PersonAlreadyEnrolled(ref scan_result_blob)
                | robonode_client::EnrollError::LogicInternal(ref scan_result_blob) => {
                    Some(error_data::ScanResultBlob(scan_result_blob.clone()).into())
                }

                robonode_client::EnrollError::FaceScanRejectedNoBlob => {
                    Some(error_data::ShouldRetry.into())
                }

                robonode_client::EnrollError::InvalidPublicKey
                | robonode_client::EnrollError::InvalidLivenessData
                | robonode_client::EnrollError::PublicKeyAlreadyUsed
                | robonode_client::EnrollError::PersonAlreadyEnrolledNoBlob
                | robonode_client::EnrollError::LogicInternalNoBlob
                | robonode_client::EnrollError::UnknownCode(_)
                | robonode_client::EnrollError::Unknown(_) => None,
            })
    }
}

#[cfg(test)]
mod tests {

    use jsonrpsee::types::ErrorObject;
    use rpc_validator_key_logic::Error as ValidatorKeyError;

    use super::*;
    use crate::errors::sign::Error as SignError;

    #[test]
    fn error_key_extraction_validator_key_extraction() {
        let error: jsonrpsee::core::Error = Error(shared::FlowBaseError::KeyExtraction(
            ValidatorKeyError::ValidatorKeyExtraction,
        ))
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
        let error: jsonrpsee::core::Error = Error(shared::FlowBaseError::KeyExtraction(
            ValidatorKeyError::MissingValidatorKey,
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":500,\"message\":\"validator key not available\",\"data\":{\"validatorKeyNotAvailable\":true}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_face_scan_rejected() {
        let error: jsonrpsee::core::Error = Error(shared::FlowBaseError::RobonodeClient(
            robonode_client::Error::Call(robonode_client::EnrollError::FaceScanRejected(
                "scan result blob".to_owned(),
            )),
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: face scan rejected\",\"data\":{\"scanResultBlob\":\"scan result blob\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_logic_internal() {
        let error: jsonrpsee::core::Error = Error(shared::FlowBaseError::RobonodeClient(
            robonode_client::Error::Call(robonode_client::EnrollError::LogicInternal(
                "scan result blob".to_owned(),
            )),
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: logic internal error\",\"data\":{\"scanResultBlob\":\"scan result blob\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_other() {
        let error: jsonrpsee::core::Error = Error(shared::FlowBaseError::RobonodeClient(
            robonode_client::Error::Call(robonode_client::EnrollError::Unknown("test".to_owned())),
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
        let error: jsonrpsee::core::Error =
            Error(shared::FlowBaseError::Sign(SignError::SigningFailed)).into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":100,\"message\":\"signing failed\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
