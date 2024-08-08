//! POST `/enrollment-3d`

use serde::{Deserialize, Serialize};

use super::Client;
use crate::OpaqueBase64DataRef;

impl<RBEI> Client<RBEI>
where
    RBEI: crate::response_body_error::Inspector,
{
    /// Perform the `/enrollment-3d` call to the server.
    pub async fn enrollment_3d(&self, req: Request<'_>) -> Result<Response, crate::Error> {
        let res = self.build_post("/enrollment-3d", &req).send().await?;
        self.parse_response(res).await
    }
}

/// Input data for the `/enrollment-3d` request.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
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
/// The schema for this particular call if fucked beyond belief; without a proper API docs from
/// the FaceTec side, implementing this properly will be a waste of time, and error prone.
/// Plus, even the spec won't help - they need to fix their approach to the API design.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// FaceScan response portion.
    #[serde(flatten)]
    pub face_scan: FaceScanResponse,
    /// The external database ID that was associated with this item.
    #[serde(rename = "externalDatabaseRefID")]
    pub external_database_ref_id: String,
    /// Scan result blob.
    pub scan_result_blob: String,
    /// Whether the request was successful.
    pub success: bool,
}

/// A FaceScan-related FaceTec API response portion.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaceScanResponse {
    /// The the information about the security checks over the FaceScan data.
    pub face_scan_security_checks: FaceScanSecurityChecks,
    /// Something to do with the retry screen of the FaceTec Device SDK.
    /// TODO(#307): find more info on this parameter.
    pub retry_screen_enum_int: i64,
    /// The age group enum id that the input FaceScan was classified to.
    pub age_estimate_group_enum_int: i64,
}

/// The report on the security checks.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FaceScanSecurityChecks {
    /// The Audit Trail Image came from the same Session as the FaceScan and the Audit Trail Image Matches the User in the FaceScan.
    pub audit_trail_verification_check_succeeded: bool,
    /// The FaceScan came from a Live Human and Liveness was Proven.
    pub face_scan_liveness_check_succeeded: bool,
    /// The FaceScan was not a replay.
    pub replay_check_succeeded: bool,
    /// The Session Token was valid and not expired.
    pub session_token_check_succeeded: bool,
}

impl FaceScanSecurityChecks {
    /// Returns `true` only if all of the underlying checks are `true`.
    pub fn all_checks_succeeded(&self) -> bool {
        self.audit_trail_verification_check_succeeded
            && self.face_scan_liveness_check_succeeded
            && self.replay_check_succeeded
            && self.session_token_check_succeeded
    }
}

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{self},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::{tests::test_client, ResponseBodyError, ServerError};

    #[test]
    fn request_serialization() {
        let expected_request = serde_json::json!({
            "externalDatabaseRefID": "my_test_id",
            "faceScan": "123",
            "auditTrailImage": "456",
            "lowQualityAuditTrailImage": "789"
        });

        let actual_request = serde_json::to_value(&Request {
            external_database_ref_id: "my_test_id",
            face_scan: "123",
            audit_trail_image: "456",
            low_quality_audit_trail_image: "789",
        })
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn success_response_deserialization() {
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
                "faceScanLivenessCheckSucceeded": true,
                "replayCheckSucceeded": true,
                "sessionTokenCheckSucceeded": true
            },
            "faceTecRetryScreen": 0,
            "retryScreenEnumInt": 0,
            "scanResultBlob": "BLOOOB",
            "serverInfo": {
                "version": "9.0.5",
                "mode": "Development Only",
                "notice": "Notice"
            },
            "success": true
        });

        let response: Response = serde_json::from_value(sample_response).unwrap();
        assert_matches!(
            response,
            Response {
                external_database_ref_id,
                scan_result_blob,
                success: true,
                face_scan: FaceScanResponse {
                    age_estimate_group_enum_int: -1,
                    ..
                },
            } if external_database_ref_id == "test_external_dbref_id" && scan_result_blob == "BLOOOB"
        )
    }

    #[test]
    fn real_world_1_response_deserialization() {
        let sample_response = serde_json::json!({
            "faceScanSecurityChecks": {
                "replayCheckSucceeded": false,
                "sessionTokenCheckSucceeded": true,
                "auditTrailVerificationCheckSucceeded": true,
                "faceScanLivenessCheckSucceeded": true
            },
            "ageEstimateGroupEnumInt": 2,
            "externalDatabaseRefID": "qwe",
            "retryScreenEnumInt": 0,
            "scanResultBlob": "BLOOOB",
            "success": false,
            "wasProcessed": true,
            "callData": {
                "tid": "bd987975-4fbb-441e-b59a-b26b5fd5987b",
                "path": "/enrollment-3d",
                "date": "Jul 3, 2021 5:21:16 PM",
                "epochSecond": 1625332876,
                "requestMethod": "POST"
            },
            "additionalSessionData": { "isAdditionalDataPartiallyIncomplete": true },
            "error": false,
            "serverInfo": {
                "version": "9.3.0",
                "type": "Standard",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            }
        });

        let response: Response = serde_json::from_value(sample_response).unwrap();
        assert_matches!(
            response,
            Response {
                external_database_ref_id,
                scan_result_blob,
                success: false,
                face_scan: FaceScanResponse {
                    face_scan_security_checks: FaceScanSecurityChecks {
                        audit_trail_verification_check_succeeded: true,
                        face_scan_liveness_check_succeeded: true,
                        replay_check_succeeded: false,
                        session_token_check_succeeded: true,
                    },
                    retry_screen_enum_int: 0,
                    age_estimate_group_enum_int: 2,
                },
            } if external_database_ref_id == "qwe" && scan_result_blob == "BLOOOB"
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
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
            "scanResultBlob": "BLOOOB",
            "serverInfo": {
                "version": "9.0.5",
                "mode": "Development Only",
                "notice": "Notice"
            },
            "success": false
        });

        let expected_response: Response = serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enrollment-3d"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_response = client.enrollment_3d(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_already_enrolled_error() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
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

        let expected_response = "An enrollment already exists for this externalDatabaseRefID.";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/enrollment-3d"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_response = client.enrollment_3d(sample_request).await.unwrap_err();
        assert_matches!(actual_response, crate::Error::Server(ServerError{error_message}) if error_message == expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
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

        let client = test_client(mock_server.uri());

        let actual_error = client.enrollment_3d(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::ResponseBody(ResponseBodyError::Json{body, ..}) if body == sample_response
        );
    }
}
