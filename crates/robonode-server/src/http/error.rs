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
fn enroll(err: &op_enroll::Error) -> ErrorMessage {
    match err {
        op_enroll::Error::InvalidPublicKey => {
            ErrorMessage::new(StatusCode::BAD_REQUEST, "ENROLL_INVALID_PUBLIC_KEY")
        }
        op_enroll::Error::InvalidLivenessData(_) => {
            ErrorMessage::new(StatusCode::BAD_REQUEST, "ENROLL_INVALID_LIVENESS_DATA")
        }
        op_enroll::Error::FaceScanRejected => {
            ErrorMessage::new(StatusCode::BAD_REQUEST, "ENROLL_FACE_SCAN_REJECTED")
        }
        op_enroll::Error::PublicKeyAlreadyUsed => {
            ErrorMessage::new(StatusCode::BAD_REQUEST, "ENROLL_PUBLIC_KEY_ALREADY_USED")
        }
        op_enroll::Error::PersonAlreadyEnrolled => {
            ErrorMessage::new(StatusCode::BAD_REQUEST, "ENROLL_PERSON_ALREADY_ENROLLED")
        }
        _ => ErrorMessage::new(StatusCode::INTERNAL_SERVER_ERROR, "ENROLL_INTERNAL"),
    }
}

/// Handle authenticate rejection.
fn authenticate(err: &op_authenticate::Error) -> ErrorMessage {
    match err {
        op_authenticate::Error::InvalidLivenessData(_) => ErrorMessage::new(
            StatusCode::BAD_REQUEST,
            "AUTHENTICATE_INVALID_LIVENESS_DATA",
        ),
        op_authenticate::Error::PersonNotFound => {
            ErrorMessage::new(StatusCode::BAD_REQUEST, "AUTHENTICATE_PERSON_NOT_FOUND")
        }
        op_authenticate::Error::FaceScanRejected => {
            ErrorMessage::new(StatusCode::BAD_REQUEST, "AUTHENTICATE_FACE_SCAN_REJECTED")
        }
        op_authenticate::Error::SignatureInvalid => {
            ErrorMessage::new(StatusCode::BAD_REQUEST, "AUTHENTICATE_SIGNATURE_INVALID")
        }
        _ => ErrorMessage::new(StatusCode::INTERNAL_SERVER_ERROR, "AUTHENTICATE_INTERNAL"),
    }
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
    let code =
        StatusCode::from_u16(error_message.code).expect("Pre-defined status code should work");

    Ok(warp::reply::with_status(json, code))
}
