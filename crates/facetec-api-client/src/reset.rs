//! DELETE `/delete-database-if-less-than-10-records`

use reqwest::StatusCode;
use serde::Deserialize;

use super::Client;

impl<RBEI> Client<RBEI>
where
    RBEI: crate::response_body_error::Inspector,
{
    /// Perform the `/delete-database-if-less-than-10-records` call to the server.
    pub async fn reset(&self) -> Result<Response, crate::Error<Error>> {
        let res = self
            .build("/delete-database-if-less-than-10-records", |url| {
                self.reqwest.delete(url)
            })
            .send()
            .await?;
        match res.status() {
            StatusCode::OK => Ok(self.parse_json(res).await?),
            _ => Err(crate::Error::Call(Error::Unknown(res.text().await?))),
        }
    }
}

/// The response from `/session-token`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// Whether the database was deleted or not.
    pub did_delete_database: bool,
    /// Whether the request had any errors during the execution.
    pub error: bool,
    /// Whether the request was successful.
    pub success: bool,
}

/// The `/session-token`-specific error kind.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
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

    use crate::tests::test_client;

    use super::*;

    #[test]
    fn response_deserialization() {
        let sample_response = serde_json::json!({
            "additionalSessionData": {
                "isAdditionalDataPartiallyIncomplete": true
            },
            "callData": {
                "tid": "8qV1E1vw1AW-518a5c2a-ff75-11ea-8db5-0232fd4aba88",
                "path": "/delete-database-if-less-than-10-records",
                "date": "Sep 25, 2020 21:23:13 PM",
                "epochSecond": 1601068993,
                "requestMethod": "DELETE"
            },
            "didDeleteDatabase": true,
            "error": false,
            "success": true
        });

        let response: Response = serde_json::from_value(sample_response).unwrap();
        assert_matches!(
            response,
            Response {
                did_delete_database: true,
                error: false,
                success: true,
                ..
            }
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
                "tid": "8qV1E1vw1AW-518a5c2a-ff75-11ea-8db5-0232fd4aba88",
                "path": "/delete-database-if-less-than-10-records",
                "date": "Sep 25, 2020 21:23:13 PM",
                "epochSecond": 1601068993,
                "requestMethod": "DELETE"
            },
            "didDeleteDatabase": true,
            "error": false,
            "success": true
        });

        let expected_response: Response = serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("DELETE"))
            .and(matchers::path("/delete-database-if-less-than-10-records"))
            .and(matchers::body_bytes(vec![]))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_response = client.reset().await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_response = "Some error text";

        Mock::given(matchers::method("DELETE"))
            .and(matchers::path("/delete-database-if-less-than-10-records"))
            .and(matchers::body_bytes(vec![]))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = test_client(mock_server.uri());

        let actual_error = client.reset().await.unwrap_err();
        assert_matches!(
            actual_error,
            crate::Error::Call(Error::Unknown(error_text)) if error_text == sample_response
        );
    }
}
