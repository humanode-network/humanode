//! Core logic of the system.

use facetec_api_client::{
    Client as FaceTecClient, DBEnrollError, DBEnrollRequest, DBSearchError, DBSearchRequest,
    Enrollment3DError, Enrollment3DErrorBadRequest, Enrollment3DRequest, Error as FaceTecError,
};
use tokio::sync::Mutex;

use crate::sequence::Sequence;
use serde::{Deserialize, Serialize};

/// The inner state, to be hidden behind the mutex to ensure we don't have
/// access to it unless we lock the mutex.
pub struct Locked {
    /// The sequence number.
    pub sequence: Sequence,
    /// The client for the FaceTec Server API.
    pub facetec: FaceTecClient,
    /// The utility for signing the responses.
    pub signer: (),
}

/// The overall generic logic.
pub struct Logic {
    /// The mutex over the locked portions of the logic.
    /// This way we're ensureing the operations can only be conducted under
    /// the lock.
    pub locked: Mutex<Locked>,
}

/// The request for the enroll operation.
#[derive(Debug, Deserialize)]
pub struct EnrollRequest {
    /// The public key of the validator.
    public_key: String,
    /// The face scan that validator owner provided.
    face_scan: String,
}

/// The errors on the enroll operation.
pub enum EnrollError {
    /// This FaceScan was rejected.
    FaceScanRejected,
    /// This Public Key was already used.
    PublicKeyAlreadyUsed,
    /// This person has already enrolled into the system.
    /// It can also happen if matching returns false-positive.
    PersonAlreadyEnrolled,
    /// Internal error at server-level enrollment due to the underlying request
    /// error at the API level.
    InternalErrorEnrollment(FaceTecError<Enrollment3DError>),
    /// Internal error at server-level enrollment due to unsuccessful response,
    /// but for some other reason but the FaceScan being rejected.
    /// Rejected FaceScan is explicitly encoded via a different error condition.
    InternalErrorEnrollmentUnsuccessful,
    /// Internal error at 3D-DB search due to the underlying request
    /// error at the API level.
    InternalErrorDbSearch(FaceTecError<DBSearchError>),
    /// Internal error at 3D-DB search due to unsuccessful response.
    InternalErrorDbSearchUnsuccessful,
    /// Internal error at 3D-DB enrollment due to the underlying request
    /// error at the API level.
    InternalErrorDbEnroll(FaceTecError<DBEnrollError>),
    /// Internal error at 3D-DB enrollment due to unsuccessful response.
    InternalErrorDbEnrollUnsuccessful,
}

/// This is the error message that FaceTec server returns when it
/// encounters an `externalDatabaseRefID` that is already in use.
/// For the lack of a better option, we have to compare the error messages,
/// which is not a good idea, and there should've been a better way.
const EXTERNAL_DATABASE_REF_ID_ALREADY_IN_USE_ERROR_MESSAGE: &str =
    "An enrollment already exists for this externalDatabaseRefID.";

/// The group name at 3D DB.
const DB_GROUP_NAME: &str = "";

impl Logic {
    /// An enroll invocation handler.
    pub async fn enroll(&self, req: EnrollRequest) -> Result<(), EnrollError> {
        let unlocked = self.locked.lock().await;
        let enroll_res = unlocked
            .facetec
            .enrollment_3d(Enrollment3DRequest {
                external_database_ref_id: &req.public_key,
                face_scan: &req.face_scan,
                audit_trail_image: "TODO",
                low_quality_audit_trail_image: "TODO",
            })
            .await
            .map_err(|err| match err {
                FaceTecError::Call(Enrollment3DError::BadRequest(
                    Enrollment3DErrorBadRequest { error_message, .. },
                )) if error_message == EXTERNAL_DATABASE_REF_ID_ALREADY_IN_USE_ERROR_MESSAGE => {
                    EnrollError::PublicKeyAlreadyUsed
                }
                err => EnrollError::InternalErrorEnrollment(err),
            })?;

        if !enroll_res.success {
            if !enroll_res
                .face_scan
                .face_scan_security_checks
                .all_checks_succeeded()
            {
                return Err(EnrollError::FaceScanRejected);
            }
            return Err(EnrollError::InternalErrorEnrollmentUnsuccessful);
        }

        let search_res = unlocked
            .facetec
            .db_search(DBSearchRequest {
                external_database_ref_id: &req.public_key,
                group_name: DB_GROUP_NAME,
                min_match_level: 10,
            })
            .await
            .map_err(EnrollError::InternalErrorDbSearch)?;

        if !enroll_res.success {
            return Err(EnrollError::InternalErrorDbSearchUnsuccessful);
        }

        // If the results set is non-empty - this means that this person has
        // already enrolled with the system. It might also be a false-positive.
        if !search_res.results.is_empty() {
            return Err(EnrollError::PersonAlreadyEnrolled);
        }

        let enroll_res = unlocked
            .facetec
            .db_enroll(DBEnrollRequest {
                external_database_ref_id: &req.public_key,
                group_name: "",
            })
            .await
            .map_err(EnrollError::InternalErrorDbEnroll)?;

        if !enroll_res.success {
            return Err(EnrollError::InternalErrorDbEnrollUnsuccessful);
        }

        Ok(())
    }
}

/// The request of the authenticate operation.
#[derive(Debug, Deserialize)]
pub struct AuthenticateRequest {
    /// The FaceScan that node owner provided.
    face_scan: String,
    /// The signature of the FaceScan with the private key of the node.
    /// Proves the posession of the private key by the FaceScan bearer.
    face_scan_signature: String,
}

/// The response of the authenticate operation.
#[derive(Debug, Serialize)]
pub struct AuthenticateResponse {
    /// The public key that matched with the provided FaceScan.
    public_key: String,
    /// The signature of the public key, signed with the robonode's private key.
    /// Can be used together with the public key above to prove that this
    /// public key was vetted by the robonode and verified to be associated
    /// with a FaceScan.
    authentication_signature: String,
}

/// Errors for the authenticate operation.
pub enum AuthenticateError {
    /// This FaceScan was rejected.
    FaceScanRejected,
    /// This person was not found.
    /// Unually this means they need to enroll, but it can also happen if
    /// matching returns false-negative.
    PersonNotFound,
    /// Internal error at server-level enrollment due to the underlying request
    /// error at the API level.
    InternalErrorEnrollment(FaceTecError<Enrollment3DError>),
    /// Internal error at server-level enrollment due to unsuccessful response,
    /// but for some other reason but the FaceScan being rejected.
    /// Rejected FaceScan is explicitly encoded via a different error condition.
    InternalErrorEnrollmentUnsuccessful,
    /// Internal error at 3D-DB search due to the underlying request
    /// error at the API level.
    InternalErrorDbSearch(FaceTecError<DBSearchError>),
    /// Internal error at 3D-DB search due to unsuccessful response.
    InternalErrorDbSearchUnsuccessful,
}

impl Logic {
    /// An authenticate invocation handler.
    pub async fn authenticate(
        &self,
        req: AuthenticateRequest,
    ) -> Result<AuthenticateResponse, AuthenticateError> {
        let mut unlocked = self.locked.lock().await;

        // Bump the sequence counter.
        unlocked.sequence.inc();

        // Prepare the ID to be used for this temporary FaceScan.
        let tmp_external_database_ref_id = format!("tmp-{}", unlocked.sequence.get());

        let enroll_res = unlocked
            .facetec
            .enrollment_3d(Enrollment3DRequest {
                external_database_ref_id: &tmp_external_database_ref_id,
                face_scan: &req.face_scan,
                audit_trail_image: "TODO",
                low_quality_audit_trail_image: "TODO",
            })
            .await
            .map_err(AuthenticateError::InternalErrorEnrollment)?;

        if !enroll_res.success {
            if !enroll_res
                .face_scan
                .face_scan_security_checks
                .all_checks_succeeded()
            {
                return Err(AuthenticateError::FaceScanRejected);
            }
            return Err(AuthenticateError::InternalErrorEnrollmentUnsuccessful);
        }

        let search_res = unlocked
            .facetec
            .db_search(DBSearchRequest {
                external_database_ref_id: &tmp_external_database_ref_id,
                group_name: DB_GROUP_NAME,
                min_match_level: 10,
            })
            .await
            .map_err(AuthenticateError::InternalErrorDbSearch)?;

        if !enroll_res.success {
            return Err(AuthenticateError::InternalErrorDbSearchUnsuccessful);
        }

        // If the results set is empty - this means that this person was not
        // found in the system.
        if search_res.results.is_empty() {
            return Err(AuthenticateError::PersonNotFound);
        }

        // TODO:
        // public_key.validate(face_scan_signature)?;
        // let signed_public_key = unlocked.signer.sign(public_key);
        // return both public_key and signed_public_key
        Ok(AuthenticateResponse {
            public_key: String::new(),
            authentication_signature: String::new(),
        })
    }
}
