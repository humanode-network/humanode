//! Authenticate operation.

use facetec_api_client as ft;
use primitives_auth_ticket::{AuthTicket, OpaqueAuthTicket};
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

use super::{common::*, Logic, LogicOp, ScanResultBlob, Signer, Verifier};
use crate::logic::facetec_utils::{db_search_result_adapter, DbSearchResult};

/// The request of the authenticate operation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// The liveness data that the validator owner provided.
    pub liveness_data: OpaqueLivenessData,
    /// The signature of the liveness data with the private key of the node.
    /// Proves the possession of the private key by the liveness data bearer.
    pub liveness_data_signature: Vec<u8>,
}

/// The response of the authenticate operation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
    /// Scan result blob.
    pub scan_result_blob: ScanResultBlob,
}

/// Errors for the authenticate operation.
///
/// Allow dead code to explicitly control errors data.
#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    /// The provided opaque liveness data could not be decoded.
    InvalidLivenessData(<LivenessData as TryFrom<&'static OpaqueLivenessData>>::Error),
    /// This FaceScan was rejected.
    FaceScanRejected(ScanResultBlob),
    /// This person was not found.
    /// Unually this means they need to enroll, but it can also happen if
    /// matching returns false-negative.
    PersonNotFound(ScanResultBlob),
    /// The liveness data signature validation failed.
    /// This means that the user might've provided a signature using different
    /// keypair from what was used for the original enrollment.
    SignatureInvalid(ScanResultBlob),
    /// Internal error at server-level enrollment due to the underlying request
    /// error at the API level.
    InternalErrorEnrollment(ft::Error),
    /// Internal error at server-level enrollment due to unsuccessful response,
    /// but for some other reason but the FaceScan being rejected.
    /// Rejected FaceScan is explicitly encoded via a different error condition.
    InternalErrorEnrollmentUnsuccessful(ScanResultBlob),
    /// Internal error at 3D-DB search due to the underlying request
    /// error at the API level.
    InternalErrorDbSearch(ft::Error, ScanResultBlob),
    /// Internal error at 3D-DB search due to unsuccessful response.
    InternalErrorDbSearchUnsuccessful(ScanResultBlob),
    /// Internal error at 3D-DB search due to match-level mismatch in
    /// the search results.
    InternalErrorDbSearchMatchLevelMismatch(ScanResultBlob),
    /// Internal error at converting public key hex representation to bytes.
    InternalErrorInvalidPublicKeyHex(ScanResultBlob),
    /// Internal error at public key loading due to invalid public key.
    InternalErrorInvalidPublicKey(ScanResultBlob),
    /// Internal error at signature verification.
    InternalErrorSignatureVerificationFailed(ScanResultBlob),
    /// Internal error when signing auth ticket.
    InternalErrorAuthTicketSigningFailed(ScanResultBlob),
}

#[async_trait::async_trait]
impl<S, PK> LogicOp<Request> for Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static + Sync,
    PK: Send + Sync + for<'a> TryFrom<&'a [u8]> + Verifier<Vec<u8>> + Into<Vec<u8>>,
{
    type Response = Response;
    type Error = Error;

    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error> {
        let liveness_data =
            LivenessData::try_from(&req.liveness_data).map_err(Error::InvalidLivenessData)?;

        let mut unlocked = self.locked.lock().await;

        // Bump the sequence counter.
        unlocked.sequence.inc();
        let sequence_value = unlocked.sequence.get();

        // Prepare the ID to be used for this temporary FaceScan.
        let tmp_external_database_ref_id =
            make_tmp_external_database_ref_id(unlocked.execution_id, sequence_value);

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
            if !enroll_res
                .face_scan
                .face_scan_security_checks
                .all_checks_succeeded()
            {
                return Err(Error::FaceScanRejected(enroll_res.scan_result_blob));
            }

            return Err(Error::InternalErrorEnrollmentUnsuccessful(
                enroll_res.scan_result_blob,
            ));
        }

        let ft::enrollment3d::Response {
            scan_result_blob, ..
        } = enroll_res;

        let search_result = unlocked
            .facetec
            .db_search(ft::db_search::Request {
                external_database_ref_id: &tmp_external_database_ref_id,
                group_name: DB_GROUP_NAME,
                min_match_level: MATCH_LEVEL,
            })
            .await;

        let results = match db_search_result_adapter(search_result) {
            DbSearchResult::OtherError(err) => {
                return Err(Error::InternalErrorDbSearch(err, scan_result_blob))
            }
            DbSearchResult::NoGroupError => {
                trace!(message = "Got no-group error instead of FaceTec 3D-DB search results, assuming no results");
                vec![]
            }
            DbSearchResult::Response(search_res) => {
                trace!(message = "Got FaceTec 3D-DB search results", ?search_res);
                if !search_res.success {
                    return Err(Error::InternalErrorDbSearchUnsuccessful(scan_result_blob));
                }
                search_res.results
            }
        };

        // If the results set is empty - this means that this person was not
        // found in the system.
        let Some(found) = results.first() else {
            return Err(Error::PersonNotFound(scan_result_blob));
        };

        if found.match_level < MATCH_LEVEL {
            return Err(Error::InternalErrorDbSearchMatchLevelMismatch(
                scan_result_blob,
            ));
        }

        let public_key_bytes = match hex::decode(&found.identifier) {
            Ok(public_key_bytes) => public_key_bytes,
            Err(_) => return Err(Error::InternalErrorInvalidPublicKeyHex(scan_result_blob)),
        };

        let public_key = match PK::try_from(&public_key_bytes) {
            Ok(public_key) => public_key,
            Err(_) => return Err(Error::InternalErrorInvalidPublicKey(scan_result_blob)),
        };

        let signature_valid = match public_key
            .verify(&req.liveness_data, req.liveness_data_signature)
            .await
        {
            Ok(signature_valid) => signature_valid,
            Err(_) => {
                return Err(Error::InternalErrorSignatureVerificationFailed(
                    scan_result_blob,
                ))
            }
        };

        if !signature_valid {
            return Err(Error::SignatureInvalid(scan_result_blob));
        }

        // Prepare an authentication nonce from the sequence number.
        let authentication_nonce = make_authentication_nonce(unlocked.execution_id, sequence_value);

        // Prepare the raw auth ticket.
        let auth_ticket = AuthTicket {
            public_key: public_key.into(),
            authentication_nonce,
        };

        // Prepare an opaque auth ticket, get ready for signing.
        #[allow(clippy::needless_borrow)]
        let opaque_auth_ticket = (&auth_ticket).into();

        // Sign the auth ticket with our private key, so that later on it's possible to validate
        // this ticket was issues by us.
        let auth_ticket_signature = match unlocked.signer.sign(&opaque_auth_ticket).await {
            Ok(auth_ticket_signature) => auth_ticket_signature,
            Err(_) => {
                return Err(Error::InternalErrorAuthTicketSigningFailed(
                    scan_result_blob,
                ))
            }
        };

        Ok(Response {
            auth_ticket: opaque_auth_ticket,
            auth_ticket_signature,
            scan_result_blob,
        })
    }
}

/// Make an key to store the temporary scan at.
fn make_tmp_external_database_ref_id(execution_id: uuid::Uuid, sequence_value: u64) -> String {
    format!("tmp-{execution_id}-{sequence_value}")
}

/// Make an authentication nonce.
// TODO(#306): we don't want to expose our internal sequence number, so this value should
// be hashed, or obfuscated by other means.
fn make_authentication_nonce(execution_id: uuid::Uuid, sequence_value: u64) -> Vec<u8> {
    let mut data = Vec::from(&execution_id.as_bytes()[..]);
    data.extend_from_slice(&sequence_value.to_ne_bytes()[..]);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmp_external_database_ref_id() {
        assert_eq!(
            make_tmp_external_database_ref_id(
                uuid::Uuid::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,],),
                0
            ),
            "tmp-00000000-0000-0000-0000-000000000000-0",
        );
        assert_eq!(
            make_tmp_external_database_ref_id(
                uuid::Uuid::from_bytes([
                    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc,
                    0xfd, 0xfe, 0xff
                ],),
                123
            ),
            "tmp-f0f1f2f3-f4f5-f6f7-f8f9-fafbfcfdfeff-123",
        );
    }

    #[test]
    fn authentication_nonce() {
        assert_eq!(
            make_authentication_nonce(
                uuid::Uuid::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                0
            ),
            vec![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );
        assert_eq!(
            make_authentication_nonce(
                uuid::Uuid::from_bytes([
                    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc,
                    0xfd, 0xfe, 0xff
                ],),
                1
            ),
            vec![
                0xf0u8, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc,
                0xfd, 0xfe, 0xff, 1, 0, 0, 0, 0, 0, 0, 0
            ],
        );
    }
}
