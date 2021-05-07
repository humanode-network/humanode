//! POST `/3d-db/enroll`

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{CommonResponse, Error};

use super::Client;

impl Client {
    /// Perform the `/3d-db/enroll` call to the server.
    pub async fn db_enroll(
        &self,
        req: DBEnrollRequest<'_>,
    ) -> Result<DBEnrollResponse, Error<DBEnrollError>> {
        let url = format!("{}/3d-db/enroll", self.base_url);
        let client = reqwest::Client::new();
        let res = client.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            StatusCode::BAD_REQUEST => {
                Err(Error::Call(DBEnrollError::BadRequest(res.json().await?)))
            }
            _ => Err(Error::Call(DBEnrollError::Unknown(res.text().await?))),
        }
    }
}

/// Input data for the `/3d-db/enroll` request.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DBEnrollRequest<'a> {
    /// The ID of the pre-enrolled FaceMap to use.
    #[serde(rename = "externalDatabaseRefID")]
    pub external_database_ref_id: &'a str,
    /// The name of the group to enroll the specified FaceMap at.
    pub group_name: &'a str,
}

/// The response from `/3d-db/enroll`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DBEnrollResponse {
    /// Common response portion.
    #[serde(flatten)]
    pub common: CommonResponse,
    /// The external database ID that was used.
    #[serde(rename = "externalDatabaseRefID")]
    pub external_database_ref_id: String,
    /// Whether the request had any errors during the execution.
    pub error: bool,
    /// Whether the request was successful.
    pub success: bool,
}

/// The `/3d-db/enroll`-specific error kind.
#[derive(Error, Debug, PartialEq)]
pub enum DBEnrollError {
    /// The face scan or public key were already enrolled.
    #[error("already enrolled")]
    AlreadyEnrolled,
    /// Bad request error occured.
    #[error("bad request: {0}")]
    BadRequest(DBEnrollErrorBadRequest),
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}

/// The error kind for the `/3d-db/enroll`-specific 400 response.
#[derive(Error, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[error("bad request: {error_message}")]
pub struct DBEnrollErrorBadRequest {
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
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

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
        assert_matches!(
            response,
            DBEnrollResponse {
                external_database_ref_id,
                error: false,
                success: true,
                ..
            } if external_database_ref_id == "test_external_dbref_id"
        )
    }
    #[test]
    fn bad_request_error_response_deserialization() {
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "No entry found in the database.",
            "success": false
        });

        let response: DBEnrollErrorBadRequest = serde_json::from_value(sample_response).unwrap();
        assert_eq!(
            response,
            DBEnrollErrorBadRequest {
                error: true,
                success: false,
                error_message: "No entry found in the database.".to_owned(),
            }
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = DBEnrollRequest {
            external_database_ref_id: "my_test_id",
            group_name: "",
        };
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

        let expected_response: DBEnrollResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_response = client.db_enroll(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = DBEnrollRequest {
            external_database_ref_id: "my_test_id",
            group_name: "",
        };
        let sample_response = "Some error text";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.db_enroll(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(DBEnrollError::Unknown(error_text)) if error_text == sample_response
        );
    }

    #[tokio::test]
    async fn mock_error_bad_request() {
        let mock_server = MockServer::start().await;

        let sample_request = DBEnrollRequest {
            external_database_ref_id: "my_test_id",
            group_name: "",
        };
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "No entry found in the database.",
            "success": false
        });

        let expected_error: DBEnrollErrorBadRequest =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(400).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.db_enroll(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(DBEnrollError::BadRequest(err)) if err == expected_error
        );
    }
}
