//! GET `/session-token`

use serde::Deserialize;

use super::Client;

impl<RBEI> Client<RBEI>
where
    RBEI: crate::response_body_error::Inspector,
{
    /// Perform the `/session-token` call to the server.
    pub async fn session_token(&self) -> Result<Response, crate::Error> {
        let res = self.build_get("/session-token").send().await?;
        self.parse_response(res).await
    }
}

/// The response from `/session-token`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// The Session Token.
    pub session_token: String,
    /// Whether the request was successful.
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{self},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{tests::test_client, ResponseBodyError};

    use super::*;

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": true
            },
            "callData": {
                "tid": "6vbfRTI0IAW-2b1e1d84-cf3d-11eb-86b0-0232fd4aba88",
                "path": "/session-token",
                "date": "Jun 17, 2021 07:25:18 AM",
                "epochSecond": 1623914718,
                "requestMethod": "GET"
            },
            "error": false,
            "serverInfo": {
                "version": "9.0.0-SNAPSHOT",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            },
            "sessionToken": "the session token",
            "success": true
        });

        let response: Response = serde_json::from_value(sample_response).unwrap();
        assert_matches!(
            response,
            Response {
                session_token,
                success: true,
                ..
            } if session_token == "the session token"
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_response = serde_json::json!({
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": true
            },
            "callData": {
                "tid": "6vbfRTI0IAW-2b1e1d84-cf3d-11eb-86b0-0232fd4aba88",
                "path": "/session-token",
                "date": "Jun 17, 2021 07:25:18 AM",
                "epochSecond": 1623914718,
                "requestMethod": "GET"
            },
            "error": false,
            "serverInfo": {
                "version": "9.0.0-SNAPSHOT",
                "mode": "Development Only",
                "notice": "You should only be reading this if you are in server-side code.  Please make sure you do not allow the FaceTec Server to be called from the public internet."
            },
            "sessionToken": "the session token",
            "success": true
        });

        let expected_response: Response = serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/session-token"))
            .and(matchers::body_bytes(vec![]))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_response = client.session_token().await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_response = "Some error text";

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/session-token"))
            .and(matchers::body_bytes(vec![]))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_error = client.session_token().await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::ResponseBody(ResponseBodyError::Json{body, ..}) if body == sample_response
        );
    }
}
