//! POST `/3d-db/search`

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{Error, MatchLevel};

use super::Client;

impl Client {
    /// Perform the `/3d-db/search` call to the server.
    pub async fn db_search(&self, req: DBSearchRequest<'_>) -> Result<(), Error<DBSearchError>> {
        let url = format!("{}/3d-db/search", self.base_url);
        let client = reqwest::Client::new();
        let res = client.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::CREATED => Ok(()),
            _ => Err(Error::Call(DBSearchError::Unknown(res.text().await?))),
        }
    }
}

/// Input data for the `/3d-db/search` request.
#[derive(Debug, Serialize)]
pub struct DBSearchRequest<'a> {
    /// The ID of the pre-enrolled FaceMap to search with.
    #[serde(rename = "externalDatabaseRefID")]
    external_database_ref_id: &'a str,
    /// The name of the group to search at.
    #[serde(rename = "groupName")]
    group_name: &'a str,
    /// The minimal matching level to accept into the search result.
    #[serde(rename = "minMatchLevel")]
    min_match_level: MatchLevel,
}

/// The response from `/3d-db/search`.
#[derive(Debug, Deserialize)]
pub struct DBSearchResponse {
    /// The ID of the pre-enrolled FaceMap that was used for searching
    /// as an input.
    #[serde(rename = "externalDatabaseRefID")]
    external_database_ref_id: String,
    /// Whether the request had any errors during the execution.
    error: bool,
    /// Whether the request was successful.
    success: bool,
    /// The set of all the matched entries enrolled on the group.
    results: Vec<DBSearchResponseResult>,
}

/// A single entry that matched the search request.
#[derive(Debug, Deserialize)]
pub struct DBSearchResponseResult {
    /// The external database ID associated with this entry.
    identifier: String,
    /// The level of matching this entry funfills to the input FaceMap.
    #[serde(rename = "matchLevel")]
    match_level: MatchLevel,
}

/// The `/3d-db/search`-specific error kind.
#[derive(Error, Debug)]
pub enum DBSearchError {
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
        assert!(matches!(
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
        ))
    }
}
