//! Error response handling logic and test utilities.

use serde::Deserialize;

/// A utility type assisting with decoding error response bodies.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ErrorResponse {
    /// A machine-readable code identifying the error.
    pub error_code: String,
}

impl TryFrom<String> for ErrorResponse {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&s).map_err(|_parsing_error| s)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::test_utils::mkerr;

    use super::*;

    #[test]
    fn decodes() {
        let err = mkerr("MY_ERR_CODE").to_string();
        let ErrorResponse { error_code } = err.try_into().unwrap();
        assert_eq!(error_code, "MY_ERR_CODE");
    }
}
