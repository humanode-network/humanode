//! POST `/enrollment-3d`

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{CommonResponse, Error, FaceScanResponse, OpaqueBase64DataRef};

use super::Client;

impl Client {
    /// Perform the `/enrollment-3d` call to the server.
    pub async fn enrollment_3d(
        &self,
        req: Enrollment3DRequest<'_>,
    ) -> Result<(), Error<Enrollment3DError>> {
        let url = format!("{}/enrollment-3d", self.base_url);
        let client = reqwest::Client::new();
        let res = client.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::CREATED => Ok(()),
            _ => Err(Error::Call(Enrollment3DError::Unknown(res.text().await?))),
        }
    }
}

/// Input data for the `/enrollment-3d` request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Enrollment3DRequest<'a> {
    /// The ID that the FaceTec Server will associate the data with.
    #[serde(rename = "externalDatabaseRefID")]
    external_database_ref_id: &'a str,
    /// The FaceTec 3D FaceScan to enroll into the server.
    face_scan: OpaqueBase64DataRef<'a>,
    /// The audit trail for liveness check.
    audit_trail_image: OpaqueBase64DataRef<'a>,
    /// The low quality audit trail for liveness check.
    low_quality_audit_trail_image: OpaqueBase64DataRef<'a>,
}

/// The response from `/enrollment-3d`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enrollment3DResponse {
    /// Common response portion.
    #[serde(flatten)]
    common: CommonResponse,
    /// FaceScan response portion.
    #[serde(flatten)]
    face_scan: FaceScanResponse,
    /// The external database ID that was associated with this item.
    #[serde(rename = "externalDatabaseRefID")]
    external_database_ref_id: String,
    /// Whether the request had any errors during the execution.
    error: bool,
    /// Whether the request was successful.
    success: bool,
}

/// The `/enrollment-3d`-specific error kind.
#[derive(Error, Debug)]
pub enum Enrollment3DError {
    /// The face scan or public key were already enrolled.
    #[error("already enrolled")]
    AlreadyEnrolled,
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use crate::{AdditionalSessionData, CallData};

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
        assert!(matches!(
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
        ))
    }
}
