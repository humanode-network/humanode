//! Error handle logic.

use crate::logic::{op_authenticate, op_enroll};
use serde::Serialize;
use warp::{hyper::StatusCode, Reply};

/// An API error serializable to JSON.
#[derive(Debug, Serialize)]
pub struct ErrorMessage {
    /// Status code rejection.
    pub code: u16,
    /// Message rejection.
    pub message: String,
}

impl ErrorMessage {
    /// Create a new [`ErrorMessage`].
    pub fn new(status_code: StatusCode, message: &str) -> Self {
        Self {
            code: status_code.as_u16(),
            message: message.into(),
        }
    }
}

/// Not found rejection.
fn not_found() -> ErrorMessage {
    ErrorMessage::new(StatusCode::NOT_FOUND, "NOT_FOUND")
}

/// Handle enroll rejection.
fn enroll(_err: &op_enroll::Error) -> ErrorMessage {
    ErrorMessage::new(StatusCode::BAD_REQUEST, "ENROLL_ERROR")
}

/// Handle authenticate rejection.
fn authenticate(_err: &op_authenticate::Error) -> ErrorMessage {
    ErrorMessage::new(StatusCode::BAD_REQUEST, "AUTHENTICATE_ERROR")
}

/// Internal server rejection.
fn internal_server_error() -> ErrorMessage {
    ErrorMessage::new(StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_REJECTION")
}

/// This function receives a `Rejection` and tries to return a custom
/// value, otherwise simply passes the rejection along.
pub async fn handle_rejection(
    err: warp::reject::Rejection,
) -> Result<impl Reply, std::convert::Infallible> {
    let error_message = if err.is_not_found() {
        not_found()
    } else if let Some(e) = err.find::<op_enroll::Error>() {
        enroll(e)
    } else if let Some(e) = err.find::<op_authenticate::Error>() {
        authenticate(e)
    } else {
        internal_server_error()
    };

    let json = warp::reply::json(&error_message);
    let code = StatusCode::from_u16(error_message.code).unwrap();

    Ok(warp::reply::with_status(json, code))
}
