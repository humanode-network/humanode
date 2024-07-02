//! Rejection handling logic.

use serde::Serialize;
use warp::{hyper::StatusCode, Reply};

use super::error;
use crate::logic::ScanResultBlob;

/// Error response shape that we can return for the error body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ErrorResponse {
    /// The machine-readable error code describing the error condition.
    pub error_code: &'static str,
    /// Scan result blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_result_blob: Option<ScanResultBlob>,
}

/// This function receives a `Rejection` and generates an error response.
pub async fn handle(err: warp::reject::Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let (status_code, error_code, scan_result_blob) =
        if let Some(logic_error) = err.find::<error::Logic>() {
            (
                logic_error.status_code,
                logic_error.error_code,
                logic_error.scan_result_blob.clone(),
            )
        } else {
            (StatusCode::NOT_IMPLEMENTED, "UNKNOWN_CALL", None)
        };

    let json = warp::reply::json(&ErrorResponse {
        error_code,
        scan_result_blob,
    });
    Ok(warp::reply::with_status(json, status_code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_properly() {
        let body = serde_json::to_string(&ErrorResponse {
            error_code: "MY_ERR_CODE",
            scan_result_blob: Some("scan result blob".to_owned()),
        })
        .unwrap();

        assert_eq!(
            body,
            r#"{"errorCode":"MY_ERR_CODE","scanResultBlob":"scan result blob"}"#
        );
    }
}
