//! Error response handling logic and test utilities.

use serde::Deserialize;

use crate::ScanResultBlob;

/// A utility type assisting with decoding error response bodies.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ErrorResponse {
    /// A machine-readable code identifying the error.
    pub error_code: String,
    /// Scan result blob.
    pub scan_result_blob: Option<ScanResultBlob>,
}

impl TryFrom<String> for ErrorResponse {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&s).map_err(|_parsing_error| s)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::test_utils::{mkerr, mkerr_returning_blob};

    #[test]
    fn decodes() {
        let err = mkerr("MY_ERR_CODE").to_string();
        let ErrorResponse {
            error_code,
            scan_result_blob,
        } = err.try_into().unwrap();
        assert_eq!(error_code, "MY_ERR_CODE");
        assert_eq!(scan_result_blob, None);
    }

    #[test]
    fn decodes_returning_blob() {
        let err = mkerr_returning_blob("MY_ERR_CODE", "scan result blob").to_string();
        let ErrorResponse {
            error_code,
            scan_result_blob,
        } = err.try_into().unwrap();
        assert_eq!(error_code, "MY_ERR_CODE");
        assert_eq!(scan_result_blob, Some("scan result blob".to_owned()));
    }
}
