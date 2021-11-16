//! Rejection handling logic.

use serde::Serialize;
use warp::{hyper::StatusCode, Reply};

use super::error;

/// Error response shape that we can return for the error body.
#[derive(Debug, Serialize)]
#[serde(rename = "camelCase")]
pub(super) struct ErrorResponse {
    /// The machine-readable error code describing the error condition.
    pub error_code: &'static str,
}

/// This function receives a `Rejection` and generates an error response.
pub async fn handle(err: warp::reject::Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let (status_code, error_code) = if let Some(logic_error) = err.find::<error::Logic>() {
        (logic_error.status_code, logic_error.error_code)
    } else {
        (StatusCode::NOT_IMPLEMENTED, "UNKNOWN_CALL")
    };

    let json = warp::reply::json(&ErrorResponse { error_code });
    Ok(warp::reply::with_status(json, status_code))
}
