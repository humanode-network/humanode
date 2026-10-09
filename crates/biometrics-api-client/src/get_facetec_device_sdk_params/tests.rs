use assert_matches::assert_matches;
use serde_json::json;
use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

use super::*;

#[test]
fn response_deserialization() {
    let sample_response = json!({
        "publicFaceMapEncryptionKey": "my encryption key",
        "deviceKeyIdentifier": "my device key identifier",
    });

    let response: GetFacetecDeviceSdkParamsResponse =
        serde_json::from_value(sample_response).unwrap();

    let mut expected_response = GetFacetecDeviceSdkParamsResponse::default();
    expected_response.insert(
        "publicFaceMapEncryptionKey".to_owned(),
        json!("my encryption key"),
    );
    expected_response.insert(
        "deviceKeyIdentifier".to_owned(),
        json!("my device key identifier"),
    );

    assert_eq!(response, expected_response)
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
