//! Client API for the Humanode's Bioauth Robonode.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{Client, Error};

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
    /// The match was not found, user likely needs to register first, or retry
    /// with another face scan.
    #[error("match not found")]
    MatchNotFound,
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl AuthenticateError {
    /// Parse the error response.
    fn from_response(status: StatusCode, body: String) -> Self {
        match status {
            StatusCode::NOT_FOUND => Self::MatchNotFound,
            _ => Self::Unknown(body),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    use super::*;

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
    fn response_deserialization() {
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
    async fn mock_error_match_not_found() {
        let mock_server = MockServer::start().await;

        let sample_request = AuthenticateRequest {
            liveness_data: b"dummy liveness data",
            liveness_data_signature: b"123",
        };

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/authenticate"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.authenticate(sample_request).await.unwrap_err();
        assert_matches!(actual_error, Error::Call(AuthenticateError::MatchNotFound));
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
