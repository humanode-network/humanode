//! POST `/3d-db/enroll`

use serde::{Deserialize, Serialize};

use super::Client;

impl<RBEI> Client<RBEI>
where
    RBEI: crate::response_body_error::Inspector,
{
    /// Perform the `/3d-db/enroll` call to the server.
    pub async fn db_enroll(&self, req: Request<'_>) -> Result<Response, crate::Error> {
        let res = self.build_post("/3d-db/enroll", &req).send().await?;
        self.parse_response(res).await
    }
}

/// Input data for the `/3d-db/enroll` request.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
    /// The ID of the pre-enrolled FaceMap to use.
    #[serde(rename = "externalDatabaseRefID")]
    pub external_database_ref_id: &'a str,
    /// The name of the group to enroll the specified FaceMap at.
    pub group_name: &'a str,
}

/// The response from `/3d-db/enroll`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// Whether the request was successful.
    pub success: bool,
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
            "groupName": ""
        });

        let actual_request = serde_json::to_value(&Request {
            external_database_ref_id: "my_test_id",
            group_name: "",
        })
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "success": true,
            "wasProcessed": true,
            "callData": {
                "tid": "f1f5da70-b23b-44e8-a24e-c0e8c77b5c56",
                "path": "/3d-db/enroll",
                "date": "Jul 26, 2021 3:49:24 PM",
                "epochSecond": 1627314564,
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
        assert_eq!(response, Response { success: true })
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
            external_database_ref_id: "my_test_id",
            group_name: "",
        };
        let sample_response = serde_json::json!({
            "success": true,
            "wasProcessed": true,
            "callData": {
                "tid": "f1f5da70-b23b-44e8-a24e-c0e8c77b5c56",
                "path": "/3d-db/enroll",
                "date": "Jul 26, 2021 3:49:24 PM",
                "epochSecond": 1627314564,
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

        let expected_response: Response = serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_response = client.db_enroll(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
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

        let client = test_client(mock_server.uri());

        let actual_error = client.db_enroll(sample_request).await.unwrap_err();
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
        };
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "No entry found in the database.",
            "success": false
        });

        let expected_error = "No entry found in the database.";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/enroll"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(400).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_error = client.db_enroll(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::Server(ServerError {error_message}) if error_message == expected_error
        );
    }
}
