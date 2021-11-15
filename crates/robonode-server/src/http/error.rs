//! Error handle logic.

use crate::logic::{
    op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
    op_get_public_key,
};
use serde::Serialize;
use warp::{hyper::StatusCode, Reply};

/// A logic error.
#[derive(Debug, Clone)]
pub struct Logic {
    /// The HTTP status code to serve the error response with.
    pub status_code: StatusCode,
    /// A textual code representing the rejection message.
    pub error_code: &'static str,
}

impl warp::reject::Reject for Logic {}

impl Logic {
    /// Create a new [`Logic`] error.
    pub const fn new(status_code: StatusCode, error_code: &'static str) -> Self {
        Self {
            status_code,
            error_code,
        }
    }
}

/// A kind of internal logic error occured that we don't want to expose.
const INTERNAL: Logic = Logic::new(StatusCode::INTERNAL_SERVER_ERROR, "LOGIC_INTERNAL_ERROR");

impl From<op_enroll::Error> for Logic {
    fn from(err: op_enroll::Error) -> Self {
        match err {
            op_enroll::Error::InvalidPublicKey => {
                Self::new(StatusCode::BAD_REQUEST, "ENROLL_INVALID_PUBLIC_KEY")
            }
            op_enroll::Error::InvalidLivenessData(_) => {
                Self::new(StatusCode::BAD_REQUEST, "ENROLL_INVALID_LIVENESS_DATA")
            }
            op_enroll::Error::FaceScanRejected => {
                Self::new(StatusCode::BAD_REQUEST, "ENROLL_FACE_SCAN_REJECTED")
            }
            op_enroll::Error::PublicKeyAlreadyUsed => {
                Self::new(StatusCode::BAD_REQUEST, "ENROLL_PUBLIC_KEY_ALREADY_USED")
            }
            op_enroll::Error::PersonAlreadyEnrolled => {
                Self::new(StatusCode::BAD_REQUEST, "ENROLL_PERSON_ALREADY_ENROLLED")
            }
            op_enroll::Error::InternalErrorEnrollment(_)
            | op_enroll::Error::InternalErrorEnrollmentUnsuccessful
            | op_enroll::Error::InternalErrorDbSearch(_)
            | op_enroll::Error::InternalErrorDbSearchUnsuccessful
            | op_enroll::Error::InternalErrorDbEnroll(_)
            | op_enroll::Error::InternalErrorDbEnrollUnsuccessful => INTERNAL.clone(),
        }
    }
}

impl From<op_authenticate::Error> for Logic {
    fn from(err: op_authenticate::Error) -> Self {
        match err {
            op_authenticate::Error::InvalidLivenessData(_) => Self::new(
                StatusCode::BAD_REQUEST,
                "AUTHENTICATE_INVALID_LIVENESS_DATA",
            ),
            op_authenticate::Error::PersonNotFound => {
                Self::new(StatusCode::BAD_REQUEST, "AUTHENTICATE_PERSON_NOT_FOUND")
            }
            op_authenticate::Error::FaceScanRejected => {
                Self::new(StatusCode::BAD_REQUEST, "AUTHENTICATE_FACE_SCAN_REJECTED")
            }
            op_authenticate::Error::SignatureInvalid => {
                Self::new(StatusCode::BAD_REQUEST, "AUTHENTICATE_SIGNATURE_INVALID")
            }
            op_authenticate::Error::InternalErrorEnrollment(_)
            | op_authenticate::Error::InternalErrorEnrollmentUnsuccessful
            | op_authenticate::Error::InternalErrorDbSearch(_)
            | op_authenticate::Error::InternalErrorDbSearchUnsuccessful
            | op_authenticate::Error::InternalErrorDbSearchMatchLevelMismatch
            | op_authenticate::Error::InternalErrorInvalidPublicKeyHex
            | op_authenticate::Error::InternalErrorInvalidPublicKey
            | op_authenticate::Error::InternalErrorSignatureVerificationFailed
            | op_authenticate::Error::InternalErrorAuthTicketSigningFailed => INTERNAL.clone(),
        }
    }
}

impl From<op_get_facetec_device_sdk_params::Error> for Logic {
    fn from(err: op_get_facetec_device_sdk_params::Error) -> Self {
        match err {}
    }
}

impl From<op_get_facetec_session_token::Error> for Logic {
    fn from(err: op_get_facetec_session_token::Error) -> Self {
        match err {
            op_get_facetec_session_token::Error::InternalErrorSessionToken(_)
            | op_get_facetec_session_token::Error::InternalErrorSessionTokenUnsuccessful => {
                INTERNAL.clone()
            }
        }
    }
}

impl From<op_get_public_key::Error> for Logic {
    fn from(err: op_get_public_key::Error) -> Self {
        match err {}
    }
}

/// Error response shape that we can return for the error body.
#[derive(Debug, Serialize)]
#[serde(rename = "camelCase")]
pub(super) struct Response {
    /// The machine-readable error code describing the error condition.
    pub error_code: &'static str,
}

/// This function receives a `Rejection` and generates an error response.
pub async fn handle_rejection(
    err: warp::reject::Rejection,
) -> Result<impl Reply, std::convert::Infallible> {
    let (status_code, error_response) = if let Some(logic_error) = err.find::<Logic>() {
        (
            logic_error.status_code,
            Response {
                error_code: logic_error.error_code,
            },
        )
    } else {
        (
            StatusCode::NOT_IMPLEMENTED,
            Response {
                error_code: "UNKNOWN_CALL",
            },
        )
    };

    let json = warp::reply::json(&error_response);
    Ok(warp::reply::with_status(json, status_code))
}
