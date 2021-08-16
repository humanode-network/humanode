//! Client API for the Humanode's Bioauth Robonode.

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{Client, Error};

impl Client {
    /// Perform the facetec-device-sdk-params call to the server.
    pub async fn get_facetec_device_sdk_params(
        &self,
    ) -> Result<GetFacetecDeviceSdkParamsResponse, Error<GetFacetecDeviceSdkParamsError>> {
        let url = format!("{}/facetec-device-sdk-params", self.base_url);
        let res = self.reqwest.get(url).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            _ => Err(Error::Call(GetFacetecDeviceSdkParamsError::Unknown(
                res.text().await?,
            ))),
        }
    }
}

/// Input data for the get facetec device sdk params request.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetFacetecDeviceSdkParamsResponse {
    /// The public FaceMap encription key.
    pub public_face_map_encryption_key: String,
    /// The device key identifier.
    pub device_key_identifier: String,
}

/// The get-facetec-session-token-specific error condition.
#[derive(Error, Debug, PartialEq)]
pub enum GetFacetecDeviceSdkParamsError {
    /// Some error occured.
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
            "publicFaceMapEncryptionKey": "my encryption key",
            "deviceKeyIdentifier": "my device key identifier",
        });

        let response: GetFacetecDeviceSdkParamsResponse =
            serde_json::from_value(sample_response).unwrap();
        assert_eq!(
            response,
            GetFacetecDeviceSdkParamsResponse {
                public_face_map_encryption_key: "my encryption key".into(),
                device_key_identifier: "my device key identifier".into(),
            }
        )
    }

    #[tokio::test]
    async fn mock_success() {
        let mock_server = MockServer::start().await;

        let sample_response = serde_json::json!({
            "publicFaceMapEncryptionKey": "my encryption key",
            "deviceKeyIdentifier": "my device key identifier",
        });

        let expected_response: GetFacetecDeviceSdkParamsResponse =
            serde_json::from_value(sample_response.clone()).unwrap();

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/facetec-device-sdk-params"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_response = client.get_facetec_device_sdk_params().await.unwrap();
        assert_eq!(actual_response, expected_response);
    }

    #[tokio::test]
    async fn mock_error_unknown() {
        let mock_server = MockServer::start().await;

        let sample_response = "Some error text";

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/facetec-device-sdk-params"))
            .respond_with(ResponseTemplate::new(500).set_body_string(sample_response))
            .mount(&mock_server)
            .await;

        let client = Client {
            base_url: mock_server.uri(),
            reqwest: reqwest::Client::new(),
        };

        let actual_error = client.get_facetec_device_sdk_params().await.unwrap_err();
        assert_matches!(
            actual_error,
            Error::Call(GetFacetecDeviceSdkParamsError::Unknown(error_text)) if error_text == sample_response
        );
    }
}
