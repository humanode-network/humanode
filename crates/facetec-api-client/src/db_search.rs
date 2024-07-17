//! POST `/3d-db/search`

use serde::{Deserialize, Serialize};

use super::Client;
use crate::MatchLevel;

impl<RBEI> Client<RBEI>
where
    RBEI: crate::response_body_error::Inspector,
{
    /// Perform the `/3d-db/search` call to the server.
    pub async fn db_search(&self, req: Request<'_>) -> Result<Response, crate::Error> {
        let res = self.build_post("/3d-db/search", &req).send().await?;
        self.parse_response(res).await
    }
}

/// Input data for the `/3d-db/search` request.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
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
pub struct Response {
    /// Whether the request was successful.
    pub success: bool,
    /// The set of all the matched entries enrolled on the group.
    pub results: Vec<ResponseResult>,
}

/// A single entry that matched the search request.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResponseResult {
    /// The external database ID associated with this entry.
    pub identifier: String,
    /// The level of matching this entry funfills to the input FaceMap.
    pub match_level: MatchLevel,
}

#[cfg(test)]
mod tests {
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::{tests::test_client, ResponseBodyError, ServerError};

    #[test]
    fn request_serialization() {
        let expected_request = serde_json::json!({
            "externalDatabaseRefID": "my_test_id",
            "groupName": "",
            "minMatchLevel": 10,
        });

        let actual_request = serde_json::to_value(&Request {
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

        let response: Response = serde_json::from_value(sample_response).unwrap();
        assert_matches!(
            response,
            Response {
                success: true,
                results,
                ..
            } if results.len() == 1 &&
                matches!(
                    &results[0],
                    &ResponseResult{
                        ref identifier,
                        match_level: 10,
                        ..
                    } if identifier == "test_external_dbref_id_1"
                )
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
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

        let expected_response: Response = serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/search"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_response = client.db_search(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
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

        let client = test_client(mock_server.uri());

        let actual_error = client.db_search(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::ResponseBody(ResponseBodyError::Json{body, ..}) if body == sample_response
        );
    }

    #[tokio::test]
    async fn mock_error_bad_request() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
            external_database_ref_id: "my_test_id",
            group_name: "",
            min_match_level: 10,
        };
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "No entry found in the database.",
            "success": false
        });

        let expected_error = "No entry found in the database.";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/search"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(400).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_error = client.db_search(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::Server(ServerError {error_message}) if error_message == expected_error
        );
    }

    #[tokio::test]
    async fn mock_error_bad_request_in_success() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
            external_database_ref_id: "my_test_id",
            group_name: "humanode",
            min_match_level: 10,
        };
        let sample_response = serde_json::json!({
            "errorMessage": "Tried to search a groupName when that groupName does not exist. groupName: humanode. Try adding a 3D FaceMap by calling /3d-db/enroll first.",
            "errorToString": "java.lang.Exception: Tried to search a groupName when that groupName does not exist. groupName: humanode. Try adding a 3D FaceMap by calling /3d-db/enroll first.",
            "stackTrace": "java.lang.Exception: Tried to search a groupName when that groupName does not exist. groupName: humanode. Try adding a 3D FaceMap by calling /3d-db/enroll first.\\n\\tat com.facetec.standardserver.search.SearchManager.search(SearchManager.java:64)\\n\\tat com.facetec.standardserver.processors.SearchProcessor.processRequest(SearchProcessor.java:35)\\n\\tat com.facetec.standardserver.processors.CommonProcessor.handle(CommonProcessor.java:58)\\n\\tat com.sun.net.httpserver.Filter$Chain.doFilter(Filter.java:79)\\n\\tat sun.net.httpserver.AuthFilter.doFilter(AuthFilter.java:83)\\n\\tat com.sun.net.httpserver.Filter$Chain.doFilter(Filter.java:82)\\n\\tat sun.net.httpserver.ServerImpl$Exchange$LinkHandler.handle(ServerImpl.java:675)\\n\\tat com.sun.net.httpserver.Filter$Chain.doFilter(Filter.java:79)\\n\\tat sun.net.httpserver.ServerImpl$Exchange.run(ServerImpl.java:647)\\n\\tat java.util.concurrent.ThreadPoolExecutor.runWorker(ThreadPoolExecutor.java:1149)\\n\\tat java.util.concurrent.ThreadPoolExecutor$Worker.run(ThreadPoolExecutor.java:624)\\n\\tat java.lang.Thread.run(Thread.java:748)\\n",
            "success": false,
            "wasProcessed": true,
            "error": true,
            "serverInfo": {
                "version": "9.3.0",
                "type": "Standard",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            }
        });

        let expected_error = "Tried to search a groupName when that groupName does not exist. groupName: humanode. Try adding a 3D FaceMap by calling /3d-db/enroll first.";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/search"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_error = client.db_search(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::Server(ServerError {error_message}) if error_message == expected_error
        );
    }
}
