//! Core logic of the system.

use std::{convert::TryFrom, marker::PhantomData};

use facetec_api_client as ft;
use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use tokio::sync::Mutex;
use tracing::error;

use crate::sequence::Sequence;
use serde::{Deserialize, Serialize};

/// Signer provides signatures for the data.
#[async_trait::async_trait]
pub trait Signer<S> {
    /// Signature error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Sign the provided data and return the signature, or an error if the siging fails.
    async fn sign<'a, D>(&self, data: D) -> Result<S, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// Verifier provides the verification of the data accompanied with the
/// signature or proof data.
#[async_trait::async_trait]
pub trait Verifier<S: ?Sized> {
    /// Verification error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Verify that provided data is indeed correctly signed with the provided
    /// signature.
    async fn verify<'a, D>(&self, data: D, signature: S) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// The FaceTec Device SDK params.
#[derive(Debug)]
pub struct FacetecDeviceSdkParams {
    /// The public FaceMap encription key.
    pub public_face_map_encryption_key: String,
    /// The device key identifier.
    pub device_key_identifier: String,
}

/// The inner state, to be hidden behind the mutex to ensure we don't have
/// access to it unless we lock the mutex.
pub struct Locked<S, PK> {
    /// The sequence number.
    pub sequence: Sequence,
    /// An execution ID, to be used together with sequence to guarantee unqiueness of the temporary
    /// enrollment external database IDs.
    pub execution_id: String,
    /// The client for the FaceTec Server API.
    pub facetec: ft::Client<crate::LoggingInspector>,
    /// The utility for signing the responses.
    pub signer: S,
    /// Public key type to use under the hood.
    pub public_key_type: PhantomData<PK>,
}

/// The overall generic logic.
pub struct Logic<S, PK> {
    /// The mutex over the locked portions of the logic.
    /// This way we're ensureing the operations can only be conducted under
    /// the lock.
    pub locked: Mutex<Locked<S, PK>>,
    /// The FaceTec Device SDK params to expose.
    pub facetec_device_sdk_params: FacetecDeviceSdkParams,
}

/// The request for the enroll operation.
#[derive(Debug, Deserialize)]
pub struct EnrollRequest {
    /// The public key of the validator.
    public_key: Vec<u8>,
    /// The liveness data that the validator owner provided.
    liveness_data: OpaqueLivenessData,
}

/// The errors on the enroll operation.
#[derive(Debug)]
pub enum EnrollError {
    /// The provided public key failed to load because it was invalid.
    InvalidPublicKey,
    /// The provided opaque liveness data could not be decoded.
    InvalidLivenessData(<LivenessData as TryFrom<&'static OpaqueLivenessData>>::Error),
    /// This FaceScan was rejected.
    FaceScanRejected,
    /// This Public Key was already used.
    PublicKeyAlreadyUsed,
    /// This person has already enrolled into the system.
    /// It can also happen if matching returns false-positive.
    PersonAlreadyEnrolled,
    /// Internal error at server-level enrollment due to the underlying request
    /// error at the API level.
    InternalErrorEnrollment(ft::Error<ft::enrollment3d::Error>),
    /// Internal error at server-level enrollment due to unsuccessful response,
    /// but for some other reason but the FaceScan being rejected.
    /// Rejected FaceScan is explicitly encoded via a different error condition.
    InternalErrorEnrollmentUnsuccessful,
    /// Internal error at 3D-DB search due to the underlying request
    /// error at the API level.
    InternalErrorDbSearch(ft::Error<ft::db_search::Error>),
    /// Internal error at 3D-DB search due to unsuccessful response.
    InternalErrorDbSearchUnsuccessful,
    /// Internal error at 3D-DB enrollment due to the underlying request
    /// error at the API level.
    InternalErrorDbEnroll(ft::Error<ft::db_enroll::Error>),
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
/// The match level to use throughout the code.
const MATCH_LEVEL: i64 = 10;

impl<S, PK> Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]> + AsRef<[u8]>,
{
    /// An enroll invocation handler.
    pub async fn enroll(&self, req: EnrollRequest) -> Result<(), EnrollError> {
        let public_key =
            PK::try_from(&req.public_key).map_err(|_| EnrollError::InvalidPublicKey)?;

        let liveness_data =
            LivenessData::try_from(&req.liveness_data).map_err(EnrollError::InvalidLivenessData)?;

        let public_key_hex = hex::encode(public_key);

        let unlocked = self.locked.lock().await;
        let enroll_res = unlocked
            .facetec
            .enrollment_3d(ft::enrollment3d::Request {
                external_database_ref_id: &public_key_hex,
                face_scan: &liveness_data.face_scan,
                audit_trail_image: &liveness_data.audit_trail_image,
                low_quality_audit_trail_image: &liveness_data.low_quality_audit_trail_image,
            })
            .await
            .map_err(EnrollError::InternalErrorEnrollment)?;

        if !enroll_res.success {
            error!(
                message = "Unsuccessful enroll response from FaceTec server during robonode enroll",
                ?enroll_res
            );
            if let Some(error_message) = enroll_res.error_message {
                if error_message == EXTERNAL_DATABASE_REF_ID_ALREADY_IN_USE_ERROR_MESSAGE {
                    return Err(EnrollError::PublicKeyAlreadyUsed);
                }
            } else if let Some(face_scan) = enroll_res.face_scan {
                if !face_scan.face_scan_security_checks.all_checks_succeeded() {
                    return Err(EnrollError::FaceScanRejected);
                }
            }
            return Err(EnrollError::InternalErrorEnrollmentUnsuccessful);
        }

        let search_res = unlocked
            .facetec
            .db_search(ft::db_search::Request {
                external_database_ref_id: &public_key_hex,
                group_name: DB_GROUP_NAME,
                min_match_level: MATCH_LEVEL,
            })
            .await
            .map_err(EnrollError::InternalErrorDbSearch)?;

        if !search_res.success {
            return Err(EnrollError::InternalErrorDbSearchUnsuccessful);
        }

        // If the results set is non-empty - this means that this person has
        // already enrolled with the system. It might also be a false-positive.
        if !search_res.results.is_empty() {
            return Err(EnrollError::PersonAlreadyEnrolled);
        }

        let db_enroll_res = unlocked
            .facetec
            .db_enroll(ft::db_enroll::Request {
                external_database_ref_id: &public_key_hex,
                group_name: "",
            })
            .await
            .map_err(EnrollError::InternalErrorDbEnroll)?;

        if !db_enroll_res.success {
            return Err(EnrollError::InternalErrorDbEnrollUnsuccessful);
        }

        Ok(())
    }
}

/// The request of the authenticate operation.
#[derive(Debug, Deserialize)]
pub struct AuthenticateRequest {
    /// The liveness data that the validator owner provided.
    liveness_data: OpaqueLivenessData,
    /// The signature of the liveness data with the private key of the node.
    /// Proves the posession of the private key by the liveness data bearer.
    liveness_data_signature: Vec<u8>,
}

/// The response of the authenticate operation.
#[derive(Debug, Serialize)]
pub struct AuthenticateResponse {
    /// An opaque auth ticket generated for this authentication attempt.
    /// Contains a public key that matched with the provided FaceScan and a nonce to prevent replay
    /// attacks.
    auth_ticket: OpaqueAuthTicket,
    /// The signature of the auth ticket, signed with the robonode's private key.
    /// Can be used together with the auth ticket above to prove that this
    /// auth ticket was vetted by the robonode and verified to be associated
    /// with a FaceScan.
    auth_ticket_signature: Vec<u8>,
}

/// Errors for the authenticate operation.
#[derive(Debug)]
pub enum AuthenticateError {
    /// The provided opaque liveness data could not be decoded.
    InvalidLivenessData(<LivenessData as TryFrom<&'static OpaqueLivenessData>>::Error),
    /// This FaceScan was rejected.
    FaceScanRejected,
    /// This person was not found.
    /// Unually this means they need to enroll, but it can also happen if
    /// matching returns false-negative.
    PersonNotFound,
    /// The liveness data signature validation failed.
    /// This means that the user might've provided a signature using different
    /// keypair from what was used for the original enrollment.
    SignatureInvalid,
    /// Internal error at server-level enrollment due to the underlying request
    /// error at the API level.
    InternalErrorEnrollment(ft::Error<ft::enrollment3d::Error>),
    /// Internal error at server-level enrollment due to unsuccessful response,
    /// but for some other reason but the FaceScan being rejected.
    /// Rejected FaceScan is explicitly encoded via a different error condition.
    InternalErrorEnrollmentUnsuccessful,
    /// Internal error at 3D-DB search due to the underlying request
    /// error at the API level.
    InternalErrorDbSearch(ft::Error<ft::db_search::Error>),
    /// Internal error at 3D-DB search due to unsuccessful response.
    InternalErrorDbSearchUnsuccessful,
    /// Internal error at 3D-DB search due to match-level mismatch in
    /// the search results.
    InternalErrorDbSearchMatchLevelMismatch,
    /// Internal error at converting public key hex representation to bytes.
    InternalErrorInvalidPublicKeyHex,
    /// Internal error at public key loading due to invalid public key.
    InternalErrorInvalidPublicKey,
    /// Internal error at signature verification.
    InternalErrorSignatureVerificationFailed,
    /// Internal error when signing auth ticket.
    InternalErrorAuthTicketSigningFailed,
}

impl<S, PK> Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + Sync + for<'a> TryFrom<&'a [u8]> + Verifier<Vec<u8>> + Into<Vec<u8>>,
{
    /// An authenticate invocation handler.
    pub async fn authenticate(
        &self,
        req: AuthenticateRequest,
    ) -> Result<AuthenticateResponse, AuthenticateError> {
        let liveness_data = LivenessData::try_from(&req.liveness_data)
            .map_err(AuthenticateError::InvalidLivenessData)?;

        let mut unlocked = self.locked.lock().await;

        // Bump the sequence counter.
        unlocked.sequence.inc();
        let sequence_value = unlocked.sequence.get();

        // Prepare the ID to be used for this temporary FaceScan.
        let tmp_external_database_ref_id =
            format!("tmp-{}-{}", &unlocked.execution_id, sequence_value);

        let enroll_res = unlocked
            .facetec
            .enrollment_3d(ft::enrollment3d::Request {
                external_database_ref_id: &tmp_external_database_ref_id,
                face_scan: &liveness_data.face_scan,
                audit_trail_image: &liveness_data.audit_trail_image,
                low_quality_audit_trail_image: &liveness_data.low_quality_audit_trail_image,
            })
            .await
            .map_err(AuthenticateError::InternalErrorEnrollment)?;

        if !enroll_res.success {
            error!(
                message =
                    "Unsuccessful enroll response from FaceTec server during robonode authenticate",
                ?enroll_res
            );
            if let Some(face_scan) = enroll_res.face_scan {
                if !face_scan.face_scan_security_checks.all_checks_succeeded() {
                    return Err(AuthenticateError::FaceScanRejected);
                }
            }
            return Err(AuthenticateError::InternalErrorEnrollmentUnsuccessful);
        }

        let search_res = unlocked
            .facetec
            .db_search(ft::db_search::Request {
                external_database_ref_id: &tmp_external_database_ref_id,
                group_name: DB_GROUP_NAME,
                min_match_level: MATCH_LEVEL,
            })
            .await
            .map_err(AuthenticateError::InternalErrorDbSearch)?;

        if !search_res.success {
            return Err(AuthenticateError::InternalErrorDbSearchUnsuccessful);
        }

        // If the results set is empty - this means that this person was not
        // found in the system.
        let found = search_res
            .results
            .first()
            .ok_or(AuthenticateError::PersonNotFound)?;
        if found.match_level != MATCH_LEVEL {
            return Err(AuthenticateError::InternalErrorDbSearchMatchLevelMismatch);
        }

        let public_key_bytes = hex::decode(&found.identifier)
            .map_err(|_| AuthenticateError::InternalErrorInvalidPublicKeyHex)?;
        let public_key = PK::try_from(&public_key_bytes)
            .map_err(|_| AuthenticateError::InternalErrorInvalidPublicKey)?;

        let signature_valid = public_key
            .verify(&req.liveness_data, req.liveness_data_signature)
            .await
            .map_err(|_| AuthenticateError::InternalErrorSignatureVerificationFailed)?;

        if !signature_valid {
            return Err(AuthenticateError::SignatureInvalid);
        }

        // Prepare an authentication nonce from the sequence number.
        // TODO: we don't want to expose our internal sequence number, so this value should
        // be hashed, or obfuscated by other means.
        let authentication_nonce = Vec::from(&sequence_value.to_ne_bytes()[..]);

        // Prepare the raw auth ticket.
        let auth_ticket = AuthTicket {
            public_key: public_key.into(),
            authentication_nonce,
        };

        // Prepare an opaque auth ticket, get ready for signing.
        let opaque_auth_ticket = (&auth_ticket).into();

        // Sign the auth ticket with our private key, so that later on it's possible to validate
        // this ticket was issues by us.
        let auth_ticket_signature = unlocked
            .signer
            .sign(&opaque_auth_ticket)
            .await
            .map_err(|_| AuthenticateError::InternalErrorAuthTicketSigningFailed)?;

        Ok(AuthenticateResponse {
            auth_ticket: opaque_auth_ticket,
            auth_ticket_signature,
        })
    }
}

/// The response for the get facetec session token operation.
#[derive(Debug, Serialize)]
pub struct GetFacetecSessionTokenResponse {
    /// The session token returned by the FaceTec Server.
    session_token: String,
}

/// Errors for the get facetec session token operation.
#[derive(Debug)]
pub enum GetFacetecSessionTokenError {
    /// Internal error at session token retrieval due to the underlying request
    /// error at the API level.
    InternalErrorSessionToken(ft::Error<ft::session_token::Error>),
    /// Internal error at session token retrieval due to unsuccessful response.
    InternalErrorSessionTokenUnsuccessful,
}

impl<S, PK> Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]>,
{
    /// Get a FaceTec Session Token.
    pub async fn get_facetec_session_token(
        &self,
    ) -> Result<GetFacetecSessionTokenResponse, GetFacetecSessionTokenError> {
        let unlocked = self.locked.lock().await;

        let res = unlocked
            .facetec
            .session_token()
            .await
            .map_err(GetFacetecSessionTokenError::InternalErrorSessionToken)?;

        if !res.success {
            return Err(GetFacetecSessionTokenError::InternalErrorSessionTokenUnsuccessful);
        }

        Ok(GetFacetecSessionTokenResponse {
            session_token: res.session_token,
        })
    }
}

/// The response for the get facetec device sdk params operation.
#[derive(Debug, Serialize)]
pub struct GetFacetecDeviceSdkParamsResponse {
    /// The public FaceMap encription key.
    pub public_face_map_encryption_key: String,
    /// The device key identifier.
    pub device_key_identifier: String,
}

/// Errors for the get facetec device sdk params operation.
#[derive(Debug)]
pub enum GetFacetecDeviceSdkParamsError {}

impl<S, PK> Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]>,
{
    /// Get the FaceTec Device SDK params .
    pub async fn get_facetec_device_sdk_params(
        &self,
    ) -> Result<GetFacetecDeviceSdkParamsResponse, GetFacetecDeviceSdkParamsError> {
        Ok(GetFacetecDeviceSdkParamsResponse {
            device_key_identifier: self.facetec_device_sdk_params.device_key_identifier.clone(),
            public_face_map_encryption_key: self
                .facetec_device_sdk_params
                .public_face_map_encryption_key
                .clone(),
        })
    }
}
