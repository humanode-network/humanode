//! GET `/session-token`

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{CommonResponse, Error};

use super::Client;

impl Client {
    /// Perform the `/session-token` call to the server.
    pub async fn session_token(&self) -> Result<SessionTokenResponse, Error<SessionTokenError>> {
        let res = self.build_get("/session-token").send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            _ => Err(Error::Call(SessionTokenError::Unknown(res.text().await?))),
        }
    }
}

/// The response from `/session-token`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionTokenResponse {
    /// Common response portion.
    #[serde(flatten)]
    pub common: CommonResponse,
    /// The Session Token.
    pub session_token: String,
    /// Whether the request had any errors during the execution.
    pub error: bool,
    /// Whether the request was successful.
    pub success: bool,
}

/// The `/session-token`-specific error kind.
#[derive(Error, Debug, PartialEq)]
pub enum SessionTokenError {
    /// Some error occured. We don't really expect any though.
    #[error("unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{self},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{AdditionalSessionData, CallData};

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

        let response: SessionTokenResponse = serde_json::from_value(sample_response).unwrap();
        assert_matches!(
            response,
            SessionTokenResponse {
                session_token,
                error: false,
                success: true,
                common: CommonResponse {
                    additional_session_data: AdditionalSessionData {
                        is_additional_data_partially_incomplete: true,
                        ..
                    },
                    call_data: CallData {
                        ..
                    },
                    ..
                },
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

        let expected_response: SessionTokenResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/session-token"))
            .and(matchers::body_bytes(vec![]))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
            device_key_identifier: "my device key identifier".into(),
        };

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

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
            device_key_identifier: "my device key identifier".into(),
        };

        let actual_error = client.session_token().await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(SessionTokenError::Unknown(error_text)) if error_text == sample_response
        );
    }
}
