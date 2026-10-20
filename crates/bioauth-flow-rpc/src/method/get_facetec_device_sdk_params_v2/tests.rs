use jsonrpsee::types::ErrorObject;

use super::*;

#[test]
fn error_biometrics_api() {
    let error: jsonrpsee::core::Error = Error::BiometricsApi(biometrics_api_client::Error::Call(
        biometrics_api_client::GetFacetecDeviceSdkParamsError::Unknown("test".to_owned()),
    ))
    .into();
    let error: ErrorObject = error.into();

    let expected_error_message = r#"{"code":700,"message":"server error: unknown error: test"}"#;
    assert_eq!(
        expected_error_message,
        serde_json::to_string(&error).unwrap()
    );
}
