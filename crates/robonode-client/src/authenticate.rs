//! Client API for the Humanode's Bioauth Robonode.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{error_response::ErrorResponse, Client, Error};

impl Client {
    /// Perform the authenticate call to the server.
    pub async fn authenticate(
        &self,
        req: AuthenticateRequest<'_>,
    ) -> Result<AuthenticateResponse, Error<AuthenticateError>> {
        let url = format!("{}/authenticate", self.base_url);
        let res = self.reqwest.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            status => Err(Error::Call(AuthenticateError::from_response(
                status,
                res.text().await?,
            ))),
        }
    }
}

/// Input data for the authenticate request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest<'a> {
    /// An opaque liveness data, containing the FaceScan to match the identity with and
    /// the rest of the parameters necessary to conduct a liveness check.
    pub liveness_data: &'a [u8],
    /// The signature of the liveness data, proving the possession of the
    /// private key by the issuer of this request.
    pub liveness_data_signature: &'a [u8],
}

/// Input data for the authenticate request.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponse {
    /// An opaque auth ticket generated for this authentication attempt.
    pub auth_ticket: Box<[u8]>,
    /// The robonode signature for this opaque auth ticket.
    pub auth_ticket_signature: Box<[u8]>,
}

/// The authenticate-specific error condition.
#[derive(Error, Debug, PartialEq)]
pub enum AuthenticateError {
    /// The provided liveness data was invalid.
    #[error("invalid liveness data")]
    InvalidLivenessData,
    /// The person was not found, it is likely because they haven't enrolled first.
    #[error("person not found")]
    PersonNotFound,
    /// The face scan was rejected, this is likely due to a failed liveness check.
    #[error("face scan rejected")]
    FaceScanRejected,
    /// The signature was invalid, which means that the validator private key used for signing and
    /// the public key that the person enrolled with don't match.
    #[error("signature invalid")]
    SignatureInvalid,
    /// A logic internal error occurred on the server end.
    #[error("logic internal error")]
    LogicInternal,
    /// An error with an unknown code occurred.
    #[error("unknown error code: {0}")]
    UnknownCode(String),
    /// Some other error occurred.
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl AuthenticateError {
    /// Parse the error response.
    fn from_response(_status: StatusCode, body: String) -> Self {
        let error_code = match body.try_into() {
            Ok(ErrorResponse { error_code }) => error_code,
            Err(body) => return Self::Unknown(body),
        };
        match error_code.as_str() {
            "AUTHENTICATE_INVALID_LIVENESS_DATA" => Self::InvalidLivenessData,
            "AUTHENTICATE_PERSON_NOT_FOUND" => Self::PersonNotFound,
            "AUTHENTICATE_FACE_SCAN_REJECTED" => Self::FaceScanRejected,
            "AUTHENTICATE_SIGNATURE_INVALID" => Self::SignatureInvalid,
            "LOGIC_INTERNAL_ERROR" => Self::LogicInternal,
            _ => Self::UnknownCode(error_code),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::test_utils::mkerr;

    #[test]
    fn request_serialization() {
        let expected_request = serde_json::json!({
            "livenessData": [1, 2, 3],
            "livenessDataSignature": [4, 5, 6],
        });

        let actual_request = serde_json::to_value(&AuthenticateRequest {
            liveness_data: &[1, 2, 3],
            liveness_data_signature: &[4, 5, 6],
        })
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn response_deserialization_before_2023_05() {
        let sample_response = serde_json::json!({
            "authTicket": [1, 2, 3],
            "authTicketSignature": [4, 5, 6],
        });

        let response: AuthenticateResponse = serde_json::from_value(sample_response).unwrap();
        assert_eq!(
            response,
            AuthenticateResponse {
                auth_ticket: vec![1, 2, 3].into(),
                auth_ticket_signature: vec![4, 5, 6].into(),
            }
        )
    }

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "authTicket": [1, 2, 3],
            "authTicketSignature": [4, 5, 6],
            "scanResultBlob": "blob".to_owned(),
        });

        let response: AuthenticateResponse = serde_json::from_value(sample_response).unwrap();
        assert_eq!(
            response,
            AuthenticateResponse {
                auth_ticket: vec![1, 2, 3].into(),
                auth_ticket_signature: vec![4, 5, 6].into(),
            }
        )
    }

    #[tokio::test]
    async fn mock_success_before_2023_05() {
        let mock_server = MockServer::start().await;

        let sample_request = AuthenticateRequest {
            liveness_data: b"dummy liveness data",
            liveness_data_signature: b"123",
        };
        let sample_response = serde_json::json!({
            "authTicket": b"456",
            "authTicketSignature": b"789",
        });

        let expected_response: AuthenticateResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/authenticate"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_response = client.authenticate(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = AuthenticateRequest {
            liveness_data: b"dummy liveness data",
            liveness_data_signature: b"123",
        };
        let sample_response = serde_json::json!({
            "authTicket": b"456",
            "authTicketSignature": b"789",
            "scanResultBlob": "blob".to_owned(),
        });

        let expected_response: AuthenticateResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/authenticate"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_response = client.authenticate(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_response_before_2023_05() {
        let cases = [
            (
                StatusCode::BAD_REQUEST,
                "AUTHENTICATE_INVALID_LIVENESS_DATA",
                AuthenticateError::InvalidLivenessData,
            ),
            (
                StatusCode::NOT_FOUND,
                "AUTHENTICATE_PERSON_NOT_FOUND",
                AuthenticateError::PersonNotFound,
            ),
            (
                StatusCode::FORBIDDEN,
                "AUTHENTICATE_FACE_SCAN_REJECTED",
                AuthenticateError::FaceScanRejected,
            ),
            (
                StatusCode::FORBIDDEN,
                "AUTHENTICATE_SIGNATURE_INVALID",
                AuthenticateError::SignatureInvalid,
            ),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "LOGIC_INTERNAL_ERROR",
                AuthenticateError::LogicInternal,
            ),
            (
                StatusCode::BAD_REQUEST,
                "MY_ERR_CODE",
                AuthenticateError::UnknownCode("MY_ERR_CODE".to_owned()),
            ),
        ];

        for (status_code, body, expected_error) in cases {
            let mock_server = MockServer::start().await;

            let sample_request = AuthenticateRequest {
                liveness_data: b"dummy liveness data",
                liveness_data_signature: b"123",
            };

            let response = ResponseTemplate::new(status_code).set_body_json(mkerr(body, None));

            Mock::given(matchers::method("POST"))
                .and(matchers::path("/authenticate"))
                .and(matchers::body_json(&sample_request))
                .respond_with(response)
                .mount(&mock_server)
                .await;

            let client = Client {
                base_url: mock_server.uri(),
                reqwest: reqwest::Client::new(),
            };

            let actual_error = client.authenticate(sample_request).await.unwrap_err();
            assert_matches!(actual_error, Error::Call(err) if err == expected_error);
        }
    }

    #[tokio::test]
    async fn mock_error_response() {
        enum ResponseIncludesBlob {
            Yes,
            No,
        }

        let cases = [
            (
                StatusCode::BAD_REQUEST,
                "AUTHENTICATE_INVALID_LIVENESS_DATA",
                ResponseIncludesBlob::No,
                AuthenticateError::InvalidLivenessData,
            ),
            (
                StatusCode::NOT_FOUND,
                "AUTHENTICATE_PERSON_NOT_FOUND",
                ResponseIncludesBlob::Yes,
                AuthenticateError::PersonNotFound,
            ),
            (
                StatusCode::FORBIDDEN,
                "AUTHENTICATE_FACE_SCAN_REJECTED",
                ResponseIncludesBlob::Yes,
                AuthenticateError::FaceScanRejected,
            ),
            (
                StatusCode::FORBIDDEN,
                "AUTHENTICATE_SIGNATURE_INVALID",
                ResponseIncludesBlob::Yes,
                AuthenticateError::SignatureInvalid,
            ),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "LOGIC_INTERNAL_ERROR",
                ResponseIncludesBlob::Yes,
                AuthenticateError::LogicInternal,
            ),
            (
                StatusCode::BAD_REQUEST,
                "MY_ERR_CODE",
                ResponseIncludesBlob::No,
                AuthenticateError::UnknownCode("MY_ERR_CODE".to_owned()),
            ),
        ];

        for (status_code, body, response_includes_blob, expected_error) in cases {
            let mock_server = MockServer::start().await;

            let sample_request = AuthenticateRequest {
                liveness_data: b"dummy liveness data",
                liveness_data_signature: b"123",
            };

            let response_body = match response_includes_blob {
                ResponseIncludesBlob::Yes => mkerr(body, Some("scan result blob")),
                ResponseIncludesBlob::No => mkerr(body, None),
            };

            let response = ResponseTemplate::new(status_code).set_body_json(response_body);

            Mock::given(matchers::method("POST"))
                .and(matchers::path("/authenticate"))
                .and(matchers::body_json(&sample_request))
                .respond_with(response)
                .mount(&mock_server)
                .await;

            let client = Client {
                base_url: mock_server.uri(),
                reqwest: reqwest::Client::new(),
            };

            let actual_error = client.authenticate(sample_request).await.unwrap_err();
            assert_matches!(actual_error, Error::Call(err) if err == expected_error);
        }
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = AuthenticateRequest {
            liveness_data: b"dummy liveness data",
            liveness_data_signature: b"123",
        };
        let sample_response = "Some error text";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/authenticate"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.authenticate(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(AuthenticateError::Unknown(error_text)) if error_text == sample_response
        );
    }
}
