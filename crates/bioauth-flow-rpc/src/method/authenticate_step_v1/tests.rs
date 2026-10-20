use rpc_validator_key_logic::Error as ValidatorKeyError;

use super::*;
use crate::test_utils::{open_chain, to_json};

#[test]
fn error_key_extraction_validator_key_extraction() {
    let error = Error(error::shared::StepFlowError::KeyExtraction(
        ValidatorKeyError::ValidatorKeyExtraction,
    ));

    assert_eq!(
        to_json(error),
        r#"{"code":600,"message":"unable to extract own key"}"#
    );
}

#[test]
fn error_key_extraction_missing_validator_key() {
    let error = Error(error::shared::StepFlowError::KeyExtraction(
        ValidatorKeyError::MissingValidatorKey,
    ));

    assert_eq!(
        to_json(error),
        r#"{"code":500,"message":"validator key not available","data":{"validatorKeyNotAvailable":true}}"#
    );
}

#[test]
fn error_sign() {
    let error = Error(error::shared::StepFlowError::Sign(
        error::Sign::SigningFailed,
    ));

    assert_eq!(to_json(error), r#"{"code":100,"message":"signing failed"}"#);
}

#[test]
fn error_context() {
    let error = Error(error::shared::StepFlowError::Context(
        error::StepContext::NonceOverflow,
    ));

    assert_eq!(
        to_json(error),
        r#"{"code":800,"message":"invalid step context: nonce overflow"}"#
    );
}

#[test]
fn error_biometrics_api_person_not_found_open_chain() {
    let error = Error(error::shared::StepFlowError::BiometricsApi(
        biometrics_api_client::Error::Call(
            biometrics_api_client::AuthenticateStepError::PersonNotFound(open_chain()),
        ),
    ));

    assert_eq!(
        to_json(error),
        r#"{"code":700,"message":"server error: person not found","data":{"responseBlob":"response blob","context":"0x07000000000000000c010203"}}"#
    );
}

#[test]
fn error_biometrics_api_face_scan_rejected_closed_chain() {
    let error = Error(error::shared::StepFlowError::BiometricsApi(
        biometrics_api_client::Error::Call(
            biometrics_api_client::AuthenticateStepError::FaceScanRejected(
                biometrics_api_client::ErrorChain::Closed,
            ),
        ),
    ));

    assert_eq!(
        to_json(error),
        r#"{"code":700,"message":"server error: face scan rejected"}"#
    );
}

#[test]
fn error_biometrics_api_chain_done() {
    let error = Error(error::shared::StepFlowError::BiometricsApi(
        biometrics_api_client::Error::Call(biometrics_api_client::AuthenticateStepError::ChainDone),
    ));

    assert_eq!(
        to_json(error),
        r#"{"code":700,"message":"server error: chain done"}"#
    );
}

#[test]
fn error_biometrics_api_other() {
    let error = Error(error::shared::StepFlowError::BiometricsApi(
        biometrics_api_client::Error::Call(biometrics_api_client::AuthenticateStepError::Unknown(
            "test".to_owned(),
        )),
    ));

    assert_eq!(
        to_json(error),
        r#"{"code":700,"message":"server error: unknown error: test"}"#
    );
}
