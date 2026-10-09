use assert_matches::assert_matches;
use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

use super::*;
use crate::{
    test_utils::{mkerr, sample_error_chain},
    StepChain, StepChainLink, StepStatus,
};

fn sample_request() -> StepRequest<'static> {
    StepRequest {
        request_blob: "request blob",
        chain: StepChain::Start { public_key: b"123" },
        signature: b"signature",
    }
}

fn sample_request_json() -> serde_json::Value {
    serde_json::json!({
        "requestBlob": "request blob",
        "chain": { "tag": "start", "publicKey": "0x313233" },
        "signature": "0x7369676e6174757265",
    })
}

#[tokio::test]
async fn mock_success_continue() {
    let mock_server = MockServer::start().await;

    let sample_response = serde_json::json!({
        "responseBlob": "response blob",
        "status": "continue",
        "chain": { "nonce": 7, "ciphertext": "0x010203" },
    });

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/enroll-step"))
        .and(matchers::body_json(sample_request_json()))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
        .mount(&mock_server)
        .await;

    let client = Client {
        base_url: mock_server.uri(),
        reqwest: reqwest::Client::new(),
    };

    let actual_response = client.enroll_step(sample_request()).await.unwrap();
    assert_eq!(
        actual_response,
        EnrollStepResponse {
            response_blob: "response blob".to_owned(),
            status: StepStatus::Continue {
                chain: StepChainLink {
                    nonce: 7,
                    ciphertext: Box::new([1, 2, 3]),
                },
            },
        }
    );
}

#[tokio::test]
async fn mock_success_done() {
    let mock_server = MockServer::start().await;

    let sample_response = serde_json::json!({
        "responseBlob": "response blob",
        "status": "done",
    });

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/enroll-step"))
        .and(matchers::body_json(sample_request_json()))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
        .mount(&mock_server)
        .await;

    let client = Client {
        base_url: mock_server.uri(),
        reqwest: reqwest::Client::new(),
    };

    let actual_response = client.enroll_step(sample_request()).await.unwrap();
    assert_eq!(
        actual_response,
        EnrollStepResponse {
            response_blob: "response blob".to_owned(),
            status: StepStatus::Done(EnrollDone {}),
        }
    );
}

#[tokio::test]
async fn mock_error_response() {
    let cases = [
        (
            StatusCode::BAD_REQUEST,
            "ENROLL_INVALID_PUBLIC_KEY",
            ErrorChain::Closed,
            EnrollStepError::InvalidPublicKey(ErrorChain::Closed),
        ),
        (
            StatusCode::BAD_REQUEST,
            "ENROLL_INVALID_PUBLIC_KEY",
            sample_error_chain(),
            EnrollStepError::InvalidPublicKey(sample_error_chain()),
        ),
        (
            StatusCode::FORBIDDEN,
            "ENROLL_FACE_SCAN_REJECTED",
            ErrorChain::Closed,
            EnrollStepError::FaceScanRejected(ErrorChain::Closed),
        ),
        (
            StatusCode::FORBIDDEN,
            "ENROLL_FACE_SCAN_REJECTED",
            sample_error_chain(),
            EnrollStepError::FaceScanRejected(sample_error_chain()),
        ),
        (
            StatusCode::CONFLICT,
            "ENROLL_PUBLIC_KEY_ALREADY_USED",
            ErrorChain::Closed,
            EnrollStepError::PublicKeyAlreadyUsed(ErrorChain::Closed),
        ),
        (
            StatusCode::CONFLICT,
            "ENROLL_PUBLIC_KEY_ALREADY_USED",
            sample_error_chain(),
            EnrollStepError::PublicKeyAlreadyUsed(sample_error_chain()),
        ),
        (
            StatusCode::CONFLICT,
            "ENROLL_PERSON_ALREADY_ENROLLED",
            ErrorChain::Closed,
            EnrollStepError::PersonAlreadyEnrolled(ErrorChain::Closed),
        ),
        (
            StatusCode::CONFLICT,
            "ENROLL_PERSON_ALREADY_ENROLLED",
            sample_error_chain(),
            EnrollStepError::PersonAlreadyEnrolled(sample_error_chain()),
        ),
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "LOGIC_INTERNAL_ERROR",
            ErrorChain::Closed,
            EnrollStepError::LogicInternal(ErrorChain::Closed),
        ),
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "LOGIC_INTERNAL_ERROR",
            sample_error_chain(),
            EnrollStepError::LogicInternal(sample_error_chain()),
        ),
        (
            StatusCode::FORBIDDEN,
            "SIGNATURE_INVALID",
            ErrorChain::Closed,
            EnrollStepError::SignatureInvalid,
        ),
        (
            StatusCode::BAD_REQUEST,
            "CHAIN_INVALID",
            ErrorChain::Closed,
            EnrollStepError::ChainInvalid,
        ),
        (
            StatusCode::CONFLICT,
            "CHAIN_DONE",
            ErrorChain::Closed,
            EnrollStepError::ChainDone,
        ),
        (
            StatusCode::BAD_REQUEST,
            "NONCE_INVALID",
            ErrorChain::Closed,
            EnrollStepError::NonceInvalid,
        ),
        (
            StatusCode::BAD_REQUEST,
            "MY_ERR_CODE",
            ErrorChain::Closed,
            EnrollStepError::UnknownCode("MY_ERR_CODE".to_owned(), ErrorChain::Closed),
        ),
        (
            StatusCode::BAD_REQUEST,
            "MY_ERR_CODE",
            sample_error_chain(),
            EnrollStepError::UnknownCode("MY_ERR_CODE".to_owned(), sample_error_chain()),
        ),
    ];

    for (http_code, error_code, response_chain, expected_error) in cases {
        let mock_server = MockServer::start().await;

        let response = ResponseTemplate::new(http_code.as_u16())
            .set_body_json(mkerr(error_code, Some(response_chain)));

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enroll-step"))
            .and(matchers::body_json(sample_request_json()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.enroll_step(sample_request()).await.unwrap_err();
        assert_matches!(actual_error, Error::Call(err) if err == expected_error);
    }
}

#[tokio::test]
async fn mock_error_malformed() {
    let mock_server = MockServer::start().await;

    let sample_response =
        serde_json::json!({ "errorCode": "MY_ERR_CODE", "responseBlob": "response blob" })
            .to_string();

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/enroll-step"))
        .and(matchers::body_json(sample_request_json()))
        .respond_with(ResponseTemplate::new(400).set_body_string(sample_response.clone()))
        .mount(&mock_server)
        .await;

    let client = Client {
        base_url: mock_server.uri(),
        reqwest: reqwest::Client::new(),
    };

    let actual_error = client.enroll_step(sample_request()).await.unwrap_err();
    assert_matches!(
        actual_error,
        Error::Call(EnrollStepError::Unknown(error_text)) if error_text == sample_response
    );
}

#[tokio::test]
async fn mock_error_unknown() {
    let mock_server = MockServer::start().await;

    let sample_response = "Some error text";

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/enroll-step"))
        .and(matchers::body_json(sample_request_json()))
        .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
        .mount(&mock_server)
        .await;

    let client = Client {
        base_url: mock_server.uri(),
        reqwest: reqwest::Client::new(),
    };

    let actual_error = client.enroll_step(sample_request()).await.unwrap_err();
    assert_matches!(
        actual_error,
        Error::Call(EnrollStepError::Unknown(error_text)) if error_text == sample_response
    );
}
