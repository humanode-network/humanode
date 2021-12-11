//! Client API for the Humanode's Bioauth Robonode.

use reqwest::StatusCode;
use serde::Serialize;

use crate::{Client, Error};

impl Client {
    /// Perform the enroll call to the server.
    pub async fn enroll(&self, req: EnrollRequest<'_>) -> Result<(), Error<EnrollError>> {
        let url = format!("{}/enroll", self.base_url);
        let res = self.reqwest.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::CREATED => Ok(()),
            status => Err(Error::Call(EnrollError::from_response(
                status,
                res.text().await?,
            ))),
        }
    }
}

/// Input data for the enroll request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrollRequest<'a> {
    /// The public key to be used as an identity.
    pub public_key: &'a [u8],
    /// An opaque liveness data, containing the FaceScan to associate with the identity and
    /// the rest of the parameters necessary to conduct a liveness check.
    pub liveness_data: &'a [u8],
    /// The signature of the liveness data with the private key of the node.
    /// Proves the posession of the private key by the liveness data bearer.
    pub liveness_data_signature: &'a [u8],
}

/// The enroll-specific error condition.
#[derive(Error, Debug, PartialEq)]

pub enum EnrollError {
    /// The face scan or public key were already enrolled.
    #[error("already enrolled")]
    AlreadyEnrolled,
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl EnrollError {
    /// Parse the error response.
    fn from_response(status: StatusCode, body: String) -> Self {
        match status {
            StatusCode::CONFLICT => Self::AlreadyEnrolled,
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
            "publicKey": [4, 5, 6],
            "livenessDataSignature": [7, 8, 9],
        });

        let actual_request = serde_json::to_value(&EnrollRequest {
            liveness_data: &[1, 2, 3],
            public_key: &[4, 5, 6],
            liveness_data_signature: &[7, 8, 9],
        })
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = EnrollRequest {
            liveness_data: b"dummy liveness data",
            public_key: b"123",
            liveness_data_signature: b"signature",
        };

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        client.enroll(sample_request).await.unwrap();
    }

    #[tokio::test]
    async fn mock_error_conflict() {
        let mock_server = MockServer::start().await;

        let sample_request = EnrollRequest {
            liveness_data: b"dummy liveness data",
            liveness_data_signature: b"signature",
            public_key: b"123",
        };

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(409))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.enroll(sample_request).await.unwrap_err();
        assert_matches!(actual_error, Error::Call(EnrollError::AlreadyEnrolled));
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = EnrollRequest {
            liveness_data: b"dummy liveness data",
            liveness_data_signature: b"signature",
            public_key: b"123",
        };
        let sample_response = "Some error text";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.enroll(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(EnrollError::Unknown(error_text)) if error_text == sample_response
        );
    }
}
