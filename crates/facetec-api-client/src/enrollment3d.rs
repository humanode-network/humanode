//! POST `/enrollment-3d`

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{CommonResponse, Error, FaceScanResponse, OpaqueBase64DataRef, ServerInfo};

use super::Client;

impl Client {
    /// Perform the `/enrollment-3d` call to the server.
    pub async fn enrollment_3d(
        &self,
        req: Enrollment3DRequest<'_>,
    ) -> Result<Enrollment3DResponse, Error<Enrollment3DError>> {
        let res = self.build_post("/enrollment-3d", &req).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            StatusCode::BAD_REQUEST => Err(Error::Call(Enrollment3DError::BadRequest(
                res.json().await?,
            ))),
            _ => Err(Error::Call(Enrollment3DError::Unknown(res.text().await?))),
        }
    }
}

/// Input data for the `/enrollment-3d` request.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Enrollment3DRequest<'a> {
    /// The ID that the FaceTec Server will associate the data with.
    #[serde(rename = "externalDatabaseRefID")]
    pub external_database_ref_id: &'a str,
    /// The FaceTec 3D FaceScan to enroll into the server.
    pub face_scan: OpaqueBase64DataRef<'a>,
    /// The audit trail for liveness check.
    pub audit_trail_image: OpaqueBase64DataRef<'a>,
    /// The low quality audit trail for liveness check.
    pub low_quality_audit_trail_image: OpaqueBase64DataRef<'a>,
}

/// The response from `/enrollment-3d`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Enrollment3DResponse {
    /// Common response portion.
    #[serde(flatten)]
    pub common: CommonResponse,
    /// FaceScan response portion.
    #[serde(flatten)]
    pub face_scan: FaceScanResponse,
    /// The external database ID that was associated with this item.
    #[serde(rename = "externalDatabaseRefID")]
    pub external_database_ref_id: String,
    /// Whether the request had any errors during the execution.
    pub error: bool,
    /// Whether the request was successful.
    pub success: bool,
}

/// The `/enrollment-3d`-specific error kind.
#[derive(Error, Debug, PartialEq)]
pub enum Enrollment3DError {
    /// Bad request error occured.
    #[error("bad request: {0}")]
    BadRequest(Enrollment3DErrorBadRequest),
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}

/// The error kind for the `/enrollment-3d`-specific 400 response.
#[derive(Error, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[error("bad request: {error_message}")]
pub struct Enrollment3DErrorBadRequest {
    /// The information about the server.
    pub server_info: ServerInfo,
    /// Whether the request had any errors during the execution.
    /// Expected to always be `true` in this context.
    pub error: bool,
    /// Whether the request was successful.
    /// Expected to always be `false` in this context.
    pub success: bool,
    /// The error message.
    pub error_message: String,
}

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{self},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{AdditionalSessionData, CallData, ServerInfo};

    use super::*;

    #[test]
    fn request_serialization() {
        let expected_request = serde_json::json!({
            "externalDatabaseRefID": "my_test_id",
            "faceScan": "123",
            "auditTrailImage": "456",
            "lowQualityAuditTrailImage": "789"
        });

        let actual_request = serde_json::to_value(&Enrollment3DRequest {
            external_database_ref_id: "my_test_id",
            face_scan: "123",
            audit_trail_image: "456",
            low_quality_audit_trail_image: "789",
        })
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": false,
                "platform": "android",
                "appID": "com.facetec.sampleapp",
                "installationID": "0000000000000000",
                "deviceModel": "Pixel 4",
                "deviceSDKVersion": "9.0.2",
                "sessionID": "00000000-0000-0000-0000-000000000000",
                "userAgent": "UserAgent",
                "ipAddress": "1.2.3.4"
            },
            "ageEstimateGroupEnumInt": -1,
            "callData": {
                "tid": "AAAAAAAAAAA-00000000-0000-0000-0000-000000000000",
                "path": "/enrollment-3d",
                "date": "Jan 01, 2000 00:00:00 AM",
                "epochSecond": 946684800,
                "requestMethod": "POST"
            },
            "error": false,
            "externalDatabaseRefID": "test_external_dbref_id",
            "faceScanSecurityChecks": {
                "auditTrailVerificationCheckSucceeded": true,
                "faceScanLivenessCheckSucceeded": false,
                "replayCheckSucceeded": true,
                "sessionTokenCheckSucceeded": true
            },
            "faceTecRetryScreen": 0,
            "retryScreenEnumInt": 0,
            "serverInfo": {
                "version": "9.0.5",
                "mode": "Development Only",
                "notice": "Notice"
            },
            "success": false
        });

        let response: Enrollment3DResponse = serde_json::from_value(sample_response).unwrap();
        assert_matches!(
            response,
            Enrollment3DResponse {
                external_database_ref_id,
                error: false,
                success: false,
                face_scan: FaceScanResponse {
                    age_estimate_group_enum_int: -1,
                    ..
                },
                common: CommonResponse {
                    additional_session_data: AdditionalSessionData {
                        is_additional_data_partially_incomplete: false,
                        ..
                    },
                    call_data: CallData {
                        ..
                    },
                    ..
                },
                ..
            } if external_database_ref_id == "test_external_dbref_id"
        )
    }

    #[test]
    fn bad_request_error_response_deserialization() {
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "An enrollment already exists for this externalDatabaseRefID.",
            "success": false,
            "serverInfo": {
                "version": "9.0.0-SNAPSHOT",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            }
        });

        let response: Enrollment3DErrorBadRequest =
            serde_json::from_value(sample_response).unwrap();
        assert_eq!(
            response,
            Enrollment3DErrorBadRequest {
                error: true,
                success: false,
                server_info: ServerInfo {
                    version: "9.0.0-SNAPSHOT".to_owned(),
                    mode: "Development Only".to_owned(),
                    notice: "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet.".to_owned(),
                },
                error_message: "An enrollment already exists for this externalDatabaseRefID.".to_owned(),
            }
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = Enrollment3DRequest {
            external_database_ref_id: "my_test_id",
            face_scan: "123",
            audit_trail_image: "456",
            low_quality_audit_trail_image: "789",
        };
        let sample_response = serde_json::json!({
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": false,
                "platform": "android",
                "appID": "com.facetec.sampleapp",
                "installationID": "0000000000000000",
                "deviceModel": "Pixel 4",
                "deviceSDKVersion": "9.0.2",
                "sessionID": "00000000-0000-0000-0000-000000000000",
                "userAgent": "UserAgent",
                "ipAddress": "1.2.3.4"
            },
            "ageEstimateGroupEnumInt": -1,
            "callData": {
                "tid": "AAAAAAAAAAA-00000000-0000-0000-0000-000000000000",
                "path": "/enrollment-3d",
                "date": "Jan 01, 2000 00:00:00 AM",
                "epochSecond": 946684800,
                "requestMethod": "POST"
            },
            "error": false,
            "externalDatabaseRefID": "test_external_dbref_id",
            "faceScanSecurityChecks": {
                "auditTrailVerificationCheckSucceeded": true,
                "faceScanLivenessCheckSucceeded": false,
                "replayCheckSucceeded": true,
                "sessionTokenCheckSucceeded": true
            },
            "faceTecRetryScreen": 0,
            "retryScreenEnumInt": 0,
            "serverInfo": {
                "version": "9.0.5",
                "mode": "Development Only",
                "notice": "Notice"
            },
            "success": false
        });

        let expected_response: Enrollment3DResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enrollment-3d"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
            device_key_identifier: "my device key identifier".into(),
        };

        let actual_response = client.enrollment_3d(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = Enrollment3DRequest {
            external_database_ref_id: "my_test_id",
            face_scan: "123",
            audit_trail_image: "456",
            low_quality_audit_trail_image: "789",
        };
        let sample_response = "Some error text";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enrollment-3d"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
            device_key_identifier: "my device key identifier".into(),
        };

        let actual_error = client.enrollment_3d(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(Enrollment3DError::Unknown(error_text)) if error_text == sample_response
        );
    }

    #[tokio::test]
    async fn mock_error_bad_request() {
        let mock_server = MockServer::start().await;

        let sample_request = Enrollment3DRequest {
            external_database_ref_id: "my_test_id",
            face_scan: "123",
            audit_trail_image: "456",
            low_quality_audit_trail_image: "789",
        };
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "An enrollment already exists for this externalDatabaseRefID.",
            "success": false,
            "serverInfo": {
                "version": "9.0.0-SNAPSHOT",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            }
        });

        let expected_error: Enrollment3DErrorBadRequest =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enrollment-3d"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(400).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
            device_key_identifier: "my device key identifier".into(),
        };

        let actual_error = client.enrollment_3d(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(Enrollment3DError::BadRequest(err)) if err == expected_error
        );
    }
}
