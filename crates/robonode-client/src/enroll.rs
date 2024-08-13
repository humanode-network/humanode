//! Client API for the Humanode's Bioauth Robonode.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{error_response::ErrorResponse, Client, Error, ScanResultBlob};

impl Client {
    /// Perform the enroll call to the server.
    pub async fn enroll(
        &self,
        req: EnrollRequest<'_>,
    ) -> Result<EnrollResponse, Error<EnrollError>> {
        let url = format!("{}/enroll", self.base_url);
        let res = self.reqwest.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::CREATED => Ok(res.json().await?),
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
    /// Proves the possession of the private key by the liveness data bearer.
    pub liveness_data_signature: &'a [u8],
}

/// Input data for the enroll response.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnrollResponse {
    /// Scan result blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_result_blob: Option<ScanResultBlob>,
}

/// The enroll-specific error condition.
#[derive(Error, Debug, PartialEq)]

pub enum EnrollError {
    /// The public key is invalid.
    #[error("invalid public key")]
    InvalidPublicKey,
    /// The liveness data is invalid.
    #[error("invalid liveness data")]
    InvalidLivenessData,
    /// The face scan was rejected.
    #[error("face scan rejected")]
    FaceScanRejected,
    /// The face scan was rejected, returned blob.
    #[error("face scan rejected, returned blob")]
    FaceScanRejectedReturnedBlob(ScanResultBlob),
    /// The public key is already used.
    #[error("public key already used")]
    PublicKeyAlreadyUsed,
    /// The person is already enrolled.
    #[error("person already enrolled")]
    PersonAlreadyEnrolled,
    /// The person is already enrolled, returned blob.
    #[error("person already enrolled, returned blob")]
    PersonAlreadyEnrolledReturnedBlob(ScanResultBlob),
    /// A logic internal error occurred on the server end.
    #[error("logic internal error")]
    LogicInternal,
    /// A logic internal error occurred on the server end, returned blob.
    #[error("logic internal error, returned blob")]
    LogicInternalReturnedBlob(ScanResultBlob),
    /// An error with an unknown code occurred.
    #[error("unknown error code: {0}")]
    UnknownCode(String),
    /// Some other error occurred.
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl EnrollError {
    /// Parse the error response.
    fn from_response(_status: StatusCode, body: String) -> Self {
        let (error_code, scan_result_blob) = match body.try_into() {
            Ok(ErrorResponse {
                error_code,
                scan_result_blob,
            }) => (error_code, scan_result_blob),
            Err(body) => return Self::Unknown(body),
        };
        match error_code.as_str() {
            "ENROLL_INVALID_PUBLIC_KEY" => Self::InvalidPublicKey,
            "ENROLL_INVALID_LIVENESS_DATA" => Self::InvalidLivenessData,
            "ENROLL_FACE_SCAN_REJECTED" => match scan_result_blob {
                None => Self::FaceScanRejected,
                Some(scan_result_blob) => Self::FaceScanRejectedReturnedBlob(scan_result_blob),
            },
            "ENROLL_PUBLIC_KEY_ALREADY_USED" => Self::PublicKeyAlreadyUsed,
            "ENROLL_PERSON_ALREADY_ENROLLED" => match scan_result_blob {
                None => Self::PersonAlreadyEnrolled,
                Some(scan_result_blob) => Self::PersonAlreadyEnrolledReturnedBlob(scan_result_blob),
            },
            "LOGIC_INTERNAL_ERROR" => match scan_result_blob {
                None => Self::LogicInternal,
                Some(scan_result_blob) => Self::LogicInternalReturnedBlob(scan_result_blob),
            },
            _ => Self::UnknownCode(error_code),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::test_utils::{mkerr, ResponseIncludesBlob};

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
    async fn mock_success_before_2023_05() {
        let mock_server = MockServer::start().await;

        let sample_request = EnrollRequest {
            liveness_data: b"dummy liveness data",
            public_key: b"123",
            liveness_data_signature: b"signature",
        };
        let sample_response = serde_json::json!({
            "scanResultBlob": "scanResultBlob"
        });

        let expected_response: EnrollResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(201).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_response = client.enroll(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = EnrollRequest {
            liveness_data: b"dummy liveness data",
            public_key: b"123",
            liveness_data_signature: b"signature",
        };
        let sample_response = serde_json::json!({
            "scanResultBlob": "blob".to_owned(),
        });

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(201).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        client.enroll(sample_request).await.unwrap();
    }

    #[tokio::test]
    async fn mock_error_response_before_2023_05() {
        let cases = [
            (
                StatusCode::BAD_REQUEST,
                "ENROLL_INVALID_PUBLIC_KEY",
                EnrollError::InvalidPublicKey,
            ),
            (
                StatusCode::BAD_REQUEST,
                "ENROLL_INVALID_LIVENESS_DATA",
                EnrollError::InvalidLivenessData,
            ),
            (
                StatusCode::FORBIDDEN,
                "ENROLL_FACE_SCAN_REJECTED",
                EnrollError::FaceScanRejected,
            ),
            (
                StatusCode::CONFLICT,
                "ENROLL_PUBLIC_KEY_ALREADY_USED",
                EnrollError::PublicKeyAlreadyUsed,
            ),
            (
                StatusCode::CONFLICT,
                "ENROLL_PERSON_ALREADY_ENROLLED",
                EnrollError::PersonAlreadyEnrolled,
            ),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "LOGIC_INTERNAL_ERROR",
                EnrollError::LogicInternal,
            ),
            (
                StatusCode::BAD_REQUEST,
                "MY_ERR_CODE",
                EnrollError::UnknownCode("MY_ERR_CODE".to_owned()),
            ),
        ];

        for (http_code, error_code, expected_error) in cases {
            let mock_server = MockServer::start().await;

            let sample_request = EnrollRequest {
                liveness_data: b"dummy liveness data",
                liveness_data_signature: b"signature",
                public_key: b"123",
            };

            let response = ResponseTemplate::new(http_code).set_body_json(mkerr(error_code, None));

            Mock::given(matchers::method("POST"))
                .and(matchers::path("/enroll"))
                .and(matchers::body_json(&sample_request))
                .respond_with(response)
                .mount(&mock_server)
                .await;

            let client = Client {
                base_url: mock_server.uri(),
                reqwest: reqwest::Client::new(),
            };

            let actual_error = client.enroll(sample_request).await.unwrap_err();
            assert_matches!(actual_error, Error::Call(err) if err == expected_error);
        }
    }

    #[tokio::test]
    async fn mock_error_response() {
        let cases = [
            (
                StatusCode::BAD_REQUEST,
                "ENROLL_INVALID_PUBLIC_KEY",
                ResponseIncludesBlob::No,
                EnrollError::InvalidPublicKey,
            ),
            (
                StatusCode::BAD_REQUEST,
                "ENROLL_INVALID_LIVENESS_DATA",
                ResponseIncludesBlob::No,
                EnrollError::InvalidLivenessData,
            ),
            (
                StatusCode::FORBIDDEN,
                "ENROLL_FACE_SCAN_REJECTED",
                ResponseIncludesBlob::Yes,
                EnrollError::FaceScanRejected,
            ),
            (
                StatusCode::CONFLICT,
                "ENROLL_PUBLIC_KEY_ALREADY_USED",
                ResponseIncludesBlob::No,
                EnrollError::PublicKeyAlreadyUsed,
            ),
            (
                StatusCode::CONFLICT,
                "ENROLL_PERSON_ALREADY_ENROLLED",
                ResponseIncludesBlob::Yes,
                EnrollError::PersonAlreadyEnrolled,
            ),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "LOGIC_INTERNAL_ERROR",
                ResponseIncludesBlob::Yes,
                EnrollError::LogicInternal,
            ),
            (
                StatusCode::BAD_REQUEST,
                "MY_ERR_CODE",
                ResponseIncludesBlob::No,
                EnrollError::UnknownCode("MY_ERR_CODE".to_owned()),
            ),
        ];

        for (http_code, error_code, response_includes_blob, expected_error) in cases {
            let mock_server = MockServer::start().await;

            let sample_request = EnrollRequest {
                liveness_data: b"dummy liveness data",
                liveness_data_signature: b"signature",
                public_key: b"123",
            };

            let response_scan_result_blob = match response_includes_blob {
                ResponseIncludesBlob::Yes => Some("scan result blob"),
                ResponseIncludesBlob::No => None,
            };

            let response = ResponseTemplate::new(http_code)
                .set_body_json(mkerr(error_code, response_scan_result_blob));

            Mock::given(matchers::method("POST"))
                .and(matchers::path("/enroll"))
                .and(matchers::body_json(&sample_request))
                .respond_with(response)
                .mount(&mock_server)
                .await;

            let client = Client {
                base_url: mock_server.uri(),
                reqwest: reqwest::Client::new(),
            };

            let actual_error = client.enroll(sample_request).await.unwrap_err();
            assert_matches!(actual_error, Error::Call(err) if err == expected_error);
        }
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
