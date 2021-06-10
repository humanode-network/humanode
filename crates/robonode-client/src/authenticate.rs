//! Client API for the Humanode's Bioauth Robonode.

use std::convert::TryFrom;

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
            StatusCode::NOT_FOUND => Err(Error::Call(AuthenticateError::MatchNotFound)),
            _ => Err(Error::Call(AuthenticateError::Unknown(res.text().await?))),
        }
    }
}

/// Input data for the authenticate request.
#[derive(Debug, Serialize)]
pub struct AuthenticateRequest<'a> {
    /// The FaceTec 3D FaceScan to associate with the identity.
    face_scan: &'a [u8],
    /// The signature of the FaceTec 3D FaceScan, proving the posession of the
    /// private key by the issuer of this request.
    face_scan_signature: &'a [u8],
}

/// Input data for the authenticate request.
#[derive(Debug, Deserialize, PartialEq)]
pub struct AuthenticateResponse {
    /// The auth ticket generated for this authentication attempt.
    pub ticket: OpaqueAuthTicket,
    /// The robonode signature for this public key and nonce.
    pub ticket_signature: Box<[u8]>,
}

/// The one-time ticket to authenticate in the network.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct OpaqueAuthTicket(Box<[u8]>);

/// The one-time ticket to authenticate in the network.
#[derive(Debug, Deserialize, PartialEq)]
pub struct AuthTicket {
    /// The public key that matched with the provided FaceTec 3D FaceScan.
    pub public_key: Box<[u8]>,
    /// Opaque one-time use value.
    /// Robonode will issues unique nonces for each authentication attempt.
    pub authentication_nonce: Box<[u8]>,
}

impl TryFrom<&OpaqueAuthTicket> for AuthTicket {
    type Error = serde_json::Error;

    fn try_from(value: &OpaqueAuthTicket) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value.0)
    }
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

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    use super::*;

    #[test]
    fn request_serialization() {
        let expected_request = serde_json::json!({
            "face_scan": [1, 2, 3],
            "face_scan_signature": [4, 5, 6],
        });

        let actual_request = serde_json::to_value(&AuthenticateRequest {
            face_scan: &[1, 2, 3],
            face_scan_signature: &[4, 5, 6],
        })
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "public_key": [1, 2, 3],
            "public_key_signature": [4, 5, 6],
        });

        let response: AuthenticateResponse = serde_json::from_value(sample_response).unwrap();
        assert_eq!(
            response,
            AuthenticateResponse {
                public_key: vec![1, 2, 3].into(),
                public_key_signature: vec![4, 5, 6].into(),
            }
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = AuthenticateRequest {
            face_scan: b"dummy face scan",
            face_scan_signature: b"123",
        };
        let sample_response = serde_json::json!({
            "public_key": b"456",
            "public_key_signature": b"789",
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
            face_scan: b"dummy face scan",
            face_scan_signature: b"123",
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
            face_scan: b"dummy face scan",
            face_scan_signature: b"123",
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
