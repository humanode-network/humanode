//! Client API for the Humanode's Bioauth Robonode.

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{Client, Error};

impl Client {
    /// Perform the facetec-session-token call to the server.
    pub async fn get_facetec_session_token(
        &self,
    ) -> Result<GetFacetecSessionTokenResponse, Error<GetFacetecSessionTokenError>> {
        let url = format!("{}/facetec-session-token", self.base_url);
        let res = self.reqwest.get(url).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            _ => Err(Error::Call(GetFacetecSessionTokenError::Unknown(
                res.text().await?,
            ))),
        }
    }
}

/// Input data for the get facetec session token request.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetFacetecSessionTokenResponse {
    /// The FaceTec Session Token.
    pub session_token: String,
}

/// The get-facetec-session-token-specific error condition.
#[derive(Error, Debug, PartialEq)]
pub enum GetFacetecSessionTokenError {
    /// Some error occurred.
    #[error("unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    use super::*;

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "sessionToken": "my session token",
        });

        let response: GetFacetecSessionTokenResponse =
            serde_json::from_value(sample_response).unwrap();
        assert_eq!(
            response,
            GetFacetecSessionTokenResponse {
                session_token: "my session token".into(),
            }
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_response = serde_json::json!({
            "sessionToken": "my session token",
        });

        let expected_response: GetFacetecSessionTokenResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/facetec-session-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_response = client.get_facetec_session_token().await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_response = "Some error text";

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/facetec-session-token"))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.get_facetec_session_token().await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(GetFacetecSessionTokenError::Unknown(error_text)) if error_text == sample_response
        );
    }
}
