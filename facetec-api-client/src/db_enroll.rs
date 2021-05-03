//! POST `/3d-db/enroll`

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::Error;

use super::Client;

impl Client {
    /// Perform the `/3d-db/enroll` call to the server.
    pub async fn db_enroll(&self, req: DBEnrollRequest<'_>) -> Result<(), Error<DBEnrollError>> {
        let url = format!("{}/3d-db/enroll", self.base_url);
        let client = reqwest::Client::new();
        let res = client.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::CREATED => Ok(()),
            _ => Err(Error::Call(DBEnrollError::Unknown(res.text().await?))),
        }
    }
}

/// Input data for the `/3d-db/enroll` request.
#[derive(Debug, Serialize)]
pub struct DBEnrollRequest<'a> {
    /// The ID of the pre-enrolled FaceMap to use.
    #[serde(rename = "externalDatabaseRefID")]
    external_database_ref_id: &'a str,
    /// The name of the group to enroll the specified FaceMap at.
    #[serde(rename = "groupName")]
    group_name: &'a str,
}

/// The response from `/3d-db/enroll`.
#[derive(Debug, Deserialize)]
pub struct DBEnrollResponse {
    /// The external database ID that was used.
    #[serde(rename = "externalDatabaseRefID")]
    external_database_ref_id: String,
    /// Whether the request had any errors during the execution.
    error: bool,
    /// Whether the request was successful.
    success: bool,
}

/// The `/3d-db/enroll`-specific error kind.
#[derive(Error, Debug)]
pub enum DBEnrollError {
    /// The face scan or public key were already enrolled.
    #[error("already enrolled")]
    AlreadyEnrolled,
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serialization() {
        let expected_request = serde_json::json!({
            "externalDatabaseRefID": "my_test_id",
            "groupName": ""
        });

        let actual_request = serde_json::to_value(&DBEnrollRequest {
            external_database_ref_id: "my_test_id",
            group_name: "",
        })
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": true
            },
            "callData": {
                "tid": "4uJgQnnkRAW-d737c7a4-ff7e-11ea-8db5-0232fd4aba88",
                "path": "/3d-db/enroll",
                "date": "Sep 25, 2020 22:31:22 PM",
                "epochSecond": 1601073082,
                "requestMethod": "POST"
            },
            "error": false,
            "externalDatabaseRefID": "test_external_dbref_id",
            "serverInfo": {
                "version": "9.0.0",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            },
            "success": true
        });

        let response: DBEnrollResponse = serde_json::from_value(sample_response).unwrap();
        assert!(matches!(
            response,
            DBEnrollResponse {
                external_database_ref_id,
                error: false,
                success: true,
                ..
            } if external_database_ref_id == "test_external_dbref_id"
        ))
    }
}
