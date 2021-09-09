//! POST `/3d-db/delete`

use serde::{Deserialize, Serialize};

use super::Client;

impl<RBEI> Client<RBEI>
where
    RBEI: crate::response_body_error::Inspector,
{
    /// Perform the `/3d-db/delete` call to the server.
    pub async fn db_delete(&self, req: Request<'_>) -> Result<Response, crate::Error> {
        let res = self.build_post("/3d-db/delete", &req).send().await?;
        self.parse_response(res).await
    }
}

/// Input data for the `/3d-db/delete` request.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
    /// The ID of the enrolled FaceMap to delete.
    pub identifier: &'a str,
    /// The name of the group to delete the specified FaceMap from.
    pub group_name: &'a str,
}

/// The response from `/3d-db/delete`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// Whether the request was successful.
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    use crate::{tests::test_client, ResponseBodyError, ServerError};

    use super::*;

    #[test]
    fn request_serialization() {
        let expected_request = serde_json::json!({
            "identifier": "my_test_id",
            "groupName": ""
        });

        let actual_request = serde_json::to_value(&Request {
            identifier: "my_test_id",
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
                "tid": "0haAzpKGLfc4fa345-ee26-11eb-86b0-0232fd4aba88",
                "path": "/3d-db/delete",
                "date": "Jul 26, 2021 15:34:37 PM",
                "epochSecond": 1627313677,
                "requestMethod": "POST"
            },
            "error": false,
            "serverInfo": {
                "version": "9.3.1-dev-2021070201",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            },
            "success": true
        });

        let response: Response = serde_json::from_value(sample_response).unwrap();
        assert_matches!(response, Response { success: true })
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
            identifier: "my_test_id",
            group_name: "",
        };
        let sample_response = serde_json::json!({
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": true
            },
            "callData": {
                "tid": "0haAzpKGLfc4fa345-ee26-11eb-86b0-0232fd4aba88",
                "path": "/3d-db/delete",
                "date": "Jul 26, 2021 15:34:37 PM",
                "epochSecond": 1627313677,
                "requestMethod": "POST"
            },
            "error": false,
            "serverInfo": {
                "version": "9.3.1-dev-2021070201",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            },
            "success": true
        });

        let expected_response: Response = serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/delete"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_response = client.db_delete(sample_request).await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
            identifier: "my_test_id",
            group_name: "",
        };
        let sample_response = "Some error text";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/delete"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_error = client.db_delete(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::ResponseBody(ResponseBodyError::Json{body, ..}) if body == sample_response
        );
    }

    #[tokio::test]
    async fn mock_error_bad_request() {
        let mock_server = MockServer::start().await;

        let sample_request = Request {
            identifier: "my_test_id",
            group_name: "",
        };
        let sample_response = serde_json::json!({
            "error": true,
            "errorMessage": "No entry found in the database.",
            "success": false
        });

        let expected_error = "No entry found in the database.";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/3d-db/delete"))
            .and(matchers::body_json(&sample_request))
            .respond_with(ResponseTemplate::new(400).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_error = client.db_delete(sample_request).await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::Server(ServerError {error_message: err}) if err == expected_error
        );
    }
}
