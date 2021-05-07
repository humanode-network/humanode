//! POST `/3d-db/search`

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{CommonResponse, Error, MatchLevel};

use super::Client;

impl Client {
    /// Perform the `/3d-db/search` call to the server.
    pub async fn db_search(
        &self,
        req: DBSearchRequest<'_>,
    ) -> Result<DBSearchResponse, Error<DBSearchError>> {
        let url = format!("{}/3d-db/search", self.base_url);
        let client = reqwest::Client::new();
        let res = client.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            StatusCode::BAD_REQUEST => {
                Err(Error::Call(DBSearchError::BadRequest(res.json().await?)))
            }
            _ => Err(Error::Call(DBSearchError::Unknown(res.text().await?))),
        }
    }
}

/// Input data for the `/3d-db/search` request.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DBSearchRequest<'a> {
    /// The ID of the pre-enrolled FaceMap to search with.
    #[serde(rename = "externalDatabaseRefID")]
    pub external_database_ref_id: &'a str,
    /// The name of the group to search at.
    pub group_name: &'a str,
    /// The minimal matching level to accept into the search result.
    pub min_match_level: MatchLevel,
}

/// The response from `/3d-db/search`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DBSearchResponse {
    /// Common response portion.
    #[serde(flatten)]
    pub common: CommonResponse,
    /// The ID of the pre-enrolled FaceMap that was used for searching
    /// as an input.
    #[serde(rename = "externalDatabaseRefID")]
    pub external_database_ref_id: String,
    /// Whether the request had any errors during the execution.
    pub error: bool,
    /// Whether the request was successful.
    pub success: bool,
    /// The set of all the matched entries enrolled on the group.
    pub results: Vec<DBSearchResponseResult>,
}

/// A single entry that matched the search request.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DBSearchResponseResult {
    /// The external database ID associated with this entry.
    pub identifier: String,
    /// The level of matching this entry funfills to the input FaceMap.
    pub match_level: MatchLevel,
}

/// The `/3d-db/search`-specific error kind.
#[derive(Error, Debug, PartialEq)]
pub enum DBSearchError {
    /// Bad request error occured.
    #[error("bad request: {0}")]
    BadRequest(DBSearchErrorBadRequest),
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}

/// The error kind for the `/3d-db/search`-specific 400 response.
#[derive(Error, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[error("bad request: {error_message}")]
pub struct DBSearchErrorBadRequest {
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
            "groupName": "",
            "minMatchLevel": 10,
        });

        let actual_request = serde_json::to_value(&DBSearchRequest {
            external_database_ref_id: "my_test_id",
            group_name: "",
            min_match_level: 10,
        })
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "results": [
                {
                    "identifier": "test_external_dbref_id_1",
                    "matchLevel": 10
                }
            ],
            "externalDatabaseRefID": "test_external_dbref_id",
            "success": true,
            "serverInfo": {
                "version": "9.0.0",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            },
            "error": false,
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": true
            },
            "callData": {
                "tid": "IbERPISdrAW-edea765f-ff7e-11ea-8db5-0232fd4aba88",
                "path": "/3d-db/search",
                "date": "Sep 25, 2020 22:32:01 PM",
                "epochSecond": 1601073121,
                "requestMethod": "POST"
            }
        });

        let response: DBSearchResponse = serde_json::from_value(sample_response).unwrap();
        assert_matches!(
            response,
            DBSearchResponse {
                ref external_database_ref_id,
                error: false,
                success: true,
                results,
                ..
            } if external_database_ref_id == "test_external_dbref_id" &&
                results.len() == 1 &&
                matches!(
                    &results[0],
                    &DBSearchResponseResult{
                        ref identifier,
                        match_level: 10,
                        ..
                    } if identifier == "test_external_dbref_id_1"
                )
        )
    }

    #[test]
    fn bad_request_error_response_deserialization() {
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "No entry found in the database.",
            "success": false
        });

        let response: DBSearchErrorBadRequest = serde_json::from_value(sample_response).unwrap();
        assert_eq!(
            response,
            DBSearchErrorBadRequest {
                error: true,
                success: false,
                error_message: "No entry found in the database.".to_owned(),
            }
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = DBSearchRequest {
            external_database_ref_id: "my_test_id",
            group_name: "",
            min_match_level: 10,
        };
        let sample_response = serde_json::json!({
            "results": [
                {
                    "identifier": "test_external_dbref_id_1",
                    "matchLevel": 10
                }
            ],
            "externalDatabaseRefID": "test_external_dbref_id",
            "success": true,
            "serverInfo": {
                "version": "9.0.0",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            },
            "error": false,
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": true
            },
            "callData": {
                "tid": "IbERPISdrAW-edea765f-ff7e-11ea-8db5-0232fd4aba88",
                "path": "/3d-db/search",
                "date": "Sep 25, 2020 22:32:01 PM",
                "epochSecond": 1601073121,
                "requestMethod": "POST"
            }
        });

        let expected_response: DBSearchResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/search"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_response = client.db_search(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = DBSearchRequest {
            external_database_ref_id: "my_test_id",
            group_name: "",
            min_match_level: 10,
        };
        let sample_response = "Some error text";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/search"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.db_search(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(DBSearchError::Unknown(error_text)) if error_text == sample_response
        );
    }

    #[tokio::test]
    async fn mock_error_bad_request() {
        let mock_server = MockServer::start().await;

        let sample_request = DBSearchRequest {
            external_database_ref_id: "my_test_id",
            group_name: "",
            min_match_level: 10,
        };
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "No entry found in the database.",
            "success": false
        });

        let expected_error: DBSearchErrorBadRequest =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/search"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(400).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.db_search(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(DBSearchError::BadRequest(err)) if err == expected_error
        );
    }
}
