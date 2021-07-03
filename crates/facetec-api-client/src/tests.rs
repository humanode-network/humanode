use crate::{response_body_error::NoopInspector, Client};

/// Create a standard test client.
pub fn test_client(base_url: String) -> Client<NoopInspector> {
    Client {
        base_url,
        reqwest: reqwest::Client::new(),
        device_key_identifier: "my device key identifier".into(),
        response_body_error_inspector: crate::response_body_error::NoopInspector,
    }
}
