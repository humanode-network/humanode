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
        chain: StepChain::Next {
            nonce: 7,
            ciphertext: &[1, 2, 3],
        },
        signature: b"signature",
    }
}

fn sample_request_json() -> serde_json::Value {
    serde_json::json!({
        "requestBlob": "request blob",
        "chain": { "tag": "next", "nonce": 7, "ciphertext": "0x010203" },
        "signature": "0x7369676e6174757265",
    })
}

#[tokio::test]
async fn mock_success_continue() {
    let mock_server = MockServer::start().await;

    let sample_response = serde_json::json!({
        "responseBlob": "response blob",
        "status": "continue",
        "chain": { "nonce": 8, "ciphertext": "0x040506" },
    });

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/authenticate-step"))
        .and(matchers::body_json(sample_request_json()))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
        .mount(&mock_server)
        .await;

    let client = Client {
        base_url: mock_server.uri(),
        reqwest: reqwest::Client::new(),
    };

    let actual_response = client.authenticate_step(sample_request()).await.unwrap();
    assert_eq!(
        actual_response,
        AuthenticateStepResponse {
            response_blob: "response blob".to_owned(),
            status: StepStatus::Continue {
                chain: StepChainLink {
                    nonce: 8,
                    ciphertext: Box::new([4, 5, 6]),
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
        "authTicket": "0x010203",
        "authTicketSignature": "0x040506",
    });

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/authenticate-step"))
        .and(matchers::body_json(sample_request_json()))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
        .mount(&mock_server)
        .await;

    let client = Client {
        base_url: mock_server.uri(),
        reqwest: reqwest::Client::new(),
    };

    let actual_response = client.authenticate_step(sample_request()).await.unwrap();
    assert_eq!(
        actual_response,
        AuthenticateStepResponse {
            response_blob: "response blob".to_owned(),
            status: StepStatus::Done(AuthenticateDone {
                auth_ticket: Box::new([1, 2, 3]),
                auth_ticket_signature: Box::new([4, 5, 6]),
            }),
        }
    );
}

#[tokio::test]
async fn mock_error_response() {
    let cases = [
        (
            StatusCode::NOT_FOUND,
            "AUTHENTICATE_PERSON_NOT_FOUND",
            ErrorChain::Closed,
            AuthenticateStepError::PersonNotFound(ErrorChain::Closed),
        ),
        (
            StatusCode::NOT_FOUND,
            "AUTHENTICATE_PERSON_NOT_FOUND",
            sample_error_chain(),
            AuthenticateStepError::PersonNotFound(sample_error_chain()),
        ),
        (
            StatusCode::FORBIDDEN,
            "AUTHENTICATE_FACE_SCAN_REJECTED",
            ErrorChain::Closed,
            AuthenticateStepError::FaceScanRejected(ErrorChain::Closed),
        ),
        (
            StatusCode::FORBIDDEN,
            "AUTHENTICATE_FACE_SCAN_REJECTED",
            sample_error_chain(),
            AuthenticateStepError::FaceScanRejected(sample_error_chain()),
        ),
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "LOGIC_INTERNAL_ERROR",
            ErrorChain::Closed,
            AuthenticateStepError::LogicInternal(ErrorChain::Closed),
        ),
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "LOGIC_INTERNAL_ERROR",
            sample_error_chain(),
            AuthenticateStepError::LogicInternal(sample_error_chain()),
        ),
        (
            StatusCode::FORBIDDEN,
            "SIGNATURE_INVALID",
            ErrorChain::Closed,
            AuthenticateStepError::SignatureInvalid,
        ),
        (
            StatusCode::BAD_REQUEST,
            "CHAIN_INVALID",
            ErrorChain::Closed,
            AuthenticateStepError::ChainInvalid,
        ),
        (
            StatusCode::CONFLICT,
            "CHAIN_DONE",
            ErrorChain::Closed,
            AuthenticateStepError::ChainDone,
        ),
        (
            StatusCode::BAD_REQUEST,
            "NONCE_INVALID",
            ErrorChain::Closed,
            AuthenticateStepError::NonceInvalid,
        ),
        (
            StatusCode::BAD_REQUEST,
            "MY_ERR_CODE",
            ErrorChain::Closed,
            AuthenticateStepError::UnknownCode("MY_ERR_CODE".to_owned(), ErrorChain::Closed),
        ),
        (
            StatusCode::BAD_REQUEST,
            "MY_ERR_CODE",
            sample_error_chain(),
            AuthenticateStepError::UnknownCode("MY_ERR_CODE".to_owned(), sample_error_chain()),
        ),
    ];

    for (http_code, error_code, response_chain, expected_error) in cases {
        let mock_server = MockServer::start().await;

        let response = ResponseTemplate::new(http_code.as_u16())
            .set_body_json(mkerr(error_code, Some(response_chain)));

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/authenticate-step"))
            .and(matchers::body_json(sample_request_json()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client
            .authenticate_step(sample_request())
            .await
            .unwrap_err();
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
        .and(matchers::path("/authenticate-step"))
        .and(matchers::body_json(sample_request_json()))
        .respond_with(ResponseTemplate::new(400).set_body_string(sample_response.clone()))
        .mount(&mock_server)
        .await;

    let client = Client {
        base_url: mock_server.uri(),
        reqwest: reqwest::Client::new(),
    };

    let actual_error = client
        .authenticate_step(sample_request())
        .await
        .unwrap_err();
    assert_matches!(
        actual_error,
        Error::Call(AuthenticateStepError::Unknown(error_text)) if error_text == sample_response
    );
}

#[tokio::test]
async fn mock_error_unknown() {
    let mock_server = MockServer::start().await;

    let sample_response = "Some error text";

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/authenticate-step"))
        .and(matchers::body_json(sample_request_json()))
        .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
        .mount(&mock_server)
        .await;

    let client = Client {
        base_url: mock_server.uri(),
        reqwest: reqwest::Client::new(),
    };

    let actual_error = client
        .authenticate_step(sample_request())
        .await
        .unwrap_err();
    assert_matches!(
        actual_error,
        Error::Call(AuthenticateStepError::Unknown(error_text)) if error_text == sample_response
    );
}
