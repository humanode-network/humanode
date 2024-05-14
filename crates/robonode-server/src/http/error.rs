//! Error handling logic.

use warp::hyper::StatusCode;

use crate::logic::{
    op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
    op_get_public_key, ScanResultBlob,
};

/// A logic error.
#[derive(Debug, Clone)]
pub struct Logic {
    /// The HTTP status code to serve the error response with.
    pub status_code: StatusCode,
    /// A textual code representing the rejection message.
    pub error_code: &'static str,
    /// Scan result blob.
    pub scan_result_blob: ScanResultBlob,
}

impl warp::reject::Reject for Logic {}

impl Logic {
    /// Create a new [`Logic`] error.
    pub const fn new(
        status_code: StatusCode,
        error_code: &'static str,
        scan_result_blob: ScanResultBlob,
    ) -> Self {
        Self {
            status_code,
            error_code,
            scan_result_blob,
        }
    }
}

/// A kind of internal logic error occured that we don't want to expose.
const INTERNAL: Logic = Logic::new(
    StatusCode::INTERNAL_SERVER_ERROR,
    "LOGIC_INTERNAL_ERROR",
    None,
);

impl From<op_enroll::Error> for Logic {
    fn from(err: op_enroll::Error) -> Self {
        match err {
            op_enroll::Error::InvalidPublicKey => {
                Self::new(StatusCode::BAD_REQUEST, "ENROLL_INVALID_PUBLIC_KEY", None)
            }
            op_enroll::Error::InvalidLivenessData(_) => Self::new(
                StatusCode::BAD_REQUEST,
                "ENROLL_INVALID_LIVENESS_DATA",
                None,
            ),
            op_enroll::Error::SignatureInvalid => {
                Self::new(StatusCode::BAD_REQUEST, "ENROLL_SIGNATURE_INVALID", None)
            }
            op_enroll::Error::FaceScanRejected(scan_result_blob) => Self::new(
                StatusCode::FORBIDDEN,
                "ENROLL_FACE_SCAN_REJECTED",
                scan_result_blob,
            ),
            op_enroll::Error::PublicKeyAlreadyUsed => {
                Self::new(StatusCode::CONFLICT, "ENROLL_PUBLIC_KEY_ALREADY_USED", None)
            }
            op_enroll::Error::PersonAlreadyEnrolled(scan_result_blob) => Self::new(
                StatusCode::CONFLICT,
                "ENROLL_PERSON_ALREADY_ENROLLED",
                scan_result_blob,
            ),
            op_enroll::Error::InternalErrorEnrollment(_)
            | op_enroll::Error::InternalErrorEnrollmentUnsuccessful
            | op_enroll::Error::InternalErrorDbSearch(_)
            | op_enroll::Error::InternalErrorDbSearchUnsuccessful
            | op_enroll::Error::InternalErrorDbEnroll(_)
            | op_enroll::Error::InternalErrorSignatureVerificationFailed
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
                None,
            ),
            op_authenticate::Error::PersonNotFound(scan_result_blob) => Self::new(
                StatusCode::NOT_FOUND,
                "AUTHENTICATE_PERSON_NOT_FOUND",
                scan_result_blob,
            ),
            op_authenticate::Error::FaceScanRejected(scan_result_blob) => Self::new(
                StatusCode::FORBIDDEN,
                "AUTHENTICATE_FACE_SCAN_REJECTED",
                scan_result_blob,
            ),
            op_authenticate::Error::SignatureInvalid(scan_result_blob) => Self::new(
                StatusCode::FORBIDDEN,
                "AUTHENTICATE_SIGNATURE_INVALID",
                scan_result_blob,
            ),
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
