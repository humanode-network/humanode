//! Authenticate operation.

use std::convert::TryFrom;

use facetec_api_client as ft;
use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use tracing::{error, trace};

use serde::{Deserialize, Serialize};

use super::{common::*, Logic, Signer, Verifier};

/// The request of the authenticate operation.
#[derive(Debug, Deserialize)]
pub struct Request {
    /// The liveness data that the validator owner provided.
    pub liveness_data: OpaqueLivenessData,
    /// The signature of the liveness data with the private key of the node.
    /// Proves the posession of the private key by the liveness data bearer.
    pub liveness_data_signature: Vec<u8>,
}

/// The response of the authenticate operation.
#[derive(Debug, Serialize)]
pub struct Response {
    /// An opaque auth ticket generated for this authentication attempt.
    /// Contains a public key that matched with the provided FaceScan and a nonce to prevent replay
    /// attacks.
    pub auth_ticket: OpaqueAuthTicket,
    /// The signature of the auth ticket, signed with the robonode's private key.
    /// Can be used together with the auth ticket above to prove that this
    /// auth ticket was vetted by the robonode and verified to be associated
    /// with a FaceScan.
    pub auth_ticket_signature: Vec<u8>,
}

/// Errors for the authenticate operation.
#[derive(Debug)]
pub enum Error {
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
    pub async fn authenticate(&self, req: Request) -> Result<Response, Error> {
        let liveness_data =
            LivenessData::try_from(&req.liveness_data).map_err(Error::InvalidLivenessData)?;

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
            .map_err(Error::InternalErrorEnrollment)?;

        trace!(message = "Got FaceTec enroll results", ?enroll_res);

        if !enroll_res.success {
            error!(
                message =
                    "Unsuccessful enroll response from FaceTec server during robonode authenticate",
                ?enroll_res
            );
            if let Some(face_scan) = enroll_res.face_scan {
                if !face_scan.face_scan_security_checks.all_checks_succeeded() {
                    return Err(Error::FaceScanRejected);
                }
            }
            return Err(Error::InternalErrorEnrollmentUnsuccessful);
        }

        let search_res = unlocked
            .facetec
            .db_search(ft::db_search::Request {
                external_database_ref_id: &tmp_external_database_ref_id,
                group_name: DB_GROUP_NAME,
                min_match_level: MATCH_LEVEL,
            })
            .await
            .map_err(Error::InternalErrorDbSearch)?;

        trace!(message = "Got FaceTec 3D-DB search results", ?search_res);

        if !search_res.success {
            return Err(Error::InternalErrorDbSearchUnsuccessful);
        }

        // If the results set is empty - this means that this person was not
        // found in the system.
        let found = search_res.results.first().ok_or(Error::PersonNotFound)?;
        if found.match_level != MATCH_LEVEL {
            return Err(Error::InternalErrorDbSearchMatchLevelMismatch);
        }

        let public_key_bytes =
            hex::decode(&found.identifier).map_err(|_| Error::InternalErrorInvalidPublicKeyHex)?;
        let public_key =
            PK::try_from(&public_key_bytes).map_err(|_| Error::InternalErrorInvalidPublicKey)?;

        let signature_valid = public_key
            .verify(&req.liveness_data, req.liveness_data_signature)
            .await
            .map_err(|_| Error::InternalErrorSignatureVerificationFailed)?;

        if !signature_valid {
            return Err(Error::SignatureInvalid);
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
            .map_err(|_| Error::InternalErrorAuthTicketSigningFailed)?;

        Ok(Response {
            auth_ticket: opaque_auth_ticket,
            auth_ticket_signature,
        })
    }
}
