//! The `enroll` method error.

use crate::error;

/// The `enroll` method error kinds.
#[derive(Debug)]
pub struct Error(pub error::shared::FlowBaseError<robonode_client::EnrollError>);

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        err.0
            .to_jsonrpsee_error::<_, error::data::ShouldRetry>(|call_error| match call_error {
                robonode_client::EnrollError::FaceScanRejectedNoBlob
                | robonode_client::EnrollError::FaceScanRejected(_) => {
                    Some(error::data::ShouldRetry)
                }

                robonode_client::EnrollError::InvalidPublicKey
                | robonode_client::EnrollError::InvalidLivenessData
                | robonode_client::EnrollError::PublicKeyAlreadyUsed
                | robonode_client::EnrollError::PersonAlreadyEnrolledNoBlob
                | robonode_client::EnrollError::PersonAlreadyEnrolled(_)
                | robonode_client::EnrollError::LogicInternalNoBlob
                | robonode_client::EnrollError::LogicInternal(_)
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

    #[test]
    fn error_key_extraction_validator_key_extraction() {
        let error: jsonrpsee::core::Error = Error(error::shared::FlowBaseError::KeyExtraction(
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
        let error: jsonrpsee::core::Error = Error(error::shared::FlowBaseError::KeyExtraction(
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
        let error: jsonrpsee::core::Error = Error(error::shared::FlowBaseError::RobonodeClient(
            robonode_client::Error::Call(robonode_client::EnrollError::FaceScanRejectedNoBlob),
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
    fn error_robonode_logic_internal() {
        let error: jsonrpsee::core::Error = Error(error::shared::FlowBaseError::RobonodeClient(
            robonode_client::Error::Call(robonode_client::EnrollError::LogicInternalNoBlob),
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: logic internal error\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_other() {
        let error: jsonrpsee::core::Error = Error(error::shared::FlowBaseError::RobonodeClient(
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
        let error: jsonrpsee::core::Error = Error(error::shared::FlowBaseError::Sign(
            error::Sign::SigningFailed,
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":100,\"message\":\"signing failed\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
