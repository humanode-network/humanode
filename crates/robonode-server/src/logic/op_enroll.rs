//! Enroll operation.

use facetec_api_client as ft;
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

use super::{common::*, Logic, LogicOp, ScanResultBlob, Signer, Verifier};
use crate::logic::facetec_utils::{db_search_result_adapter, DbSearchResult};

/// The request for the enroll operation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// The public key of the validator.
    pub public_key: Vec<u8>,
    /// The liveness data that the validator owner provided.
    pub liveness_data: OpaqueLivenessData,
    /// The signature of the liveness data with the private key of the node.
    /// Proves the possession of the private key by the liveness data bearer.
    pub liveness_data_signature: Vec<u8>,
}

/// The response for the enroll operation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// Scan result blob.
    pub scan_result_blob: ScanResultBlob,
}

/// The errors on the enroll operation.
///
/// Allow dead code to explicitly control errors data.
#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    /// The provided public key failed to load because it was invalid.
    InvalidPublicKey,
    /// The provided opaque liveness data could not be decoded.
    InvalidLivenessData(<LivenessData as TryFrom<&'static OpaqueLivenessData>>::Error),
    /// The liveness data signature validation failed.
    SignatureInvalid,
    /// This FaceScan was rejected.
    FaceScanRejected(ScanResultBlob),
    /// This Public Key was already used.
    PublicKeyAlreadyUsed,
    /// This person has already enrolled into the system.
    /// It can also happen if matching returns false-positive.
    PersonAlreadyEnrolled(ScanResultBlob),
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
    /// Internal error at 3D-DB enrollment due to the underlying request
    /// error at the API level.
    InternalErrorDbEnroll(ft::Error, ScanResultBlob),
    /// Internal error at 3D-DB enrollment due to unsuccessful response.
    InternalErrorDbEnrollUnsuccessful(ScanResultBlob),
    /// Internal error at signature verification.
    InternalErrorSignatureVerificationFailed,
}

#[async_trait::async_trait]
impl<S, PK> LogicOp<Request> for Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + Sync + for<'a> TryFrom<&'a [u8]> + AsRef<[u8]> + Verifier<Vec<u8>>,
{
    type Response = Response;
    type Error = Error;

    /// An enroll invocation handler.
    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error> {
        let public_key = PK::try_from(&req.public_key).map_err(|_| Error::InvalidPublicKey)?;

        let liveness_data =
            LivenessData::try_from(&req.liveness_data).map_err(Error::InvalidLivenessData)?;

        let signature_valid = public_key
            .verify(&req.liveness_data, req.liveness_data_signature)
            .await
            .map_err(|_| Error::InternalErrorSignatureVerificationFailed)?;

        if !signature_valid {
            return Err(Error::SignatureInvalid);
        }

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
            .map_err(|err| match err {
                ft::Error::Server(server_error)
                    if server_error.error_message
                        == EXTERNAL_DATABASE_REF_ID_ALREADY_IN_USE_ERROR_MESSAGE =>
                {
                    Error::PublicKeyAlreadyUsed
                }
                _ => Error::InternalErrorEnrollment(err),
            })?;

        trace!(message = "Got FaceTec enroll results", ?enroll_res);

        if !enroll_res.success {
            error!(
                message = "Unsuccessful enroll response from FaceTec server during robonode enroll",
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
                external_database_ref_id: &public_key_hex,
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

        // If the results set is non-empty - this means that this person has
        // already enrolled with the system. It might also be a false-positive.
        if !results.is_empty() {
            return Err(Error::PersonAlreadyEnrolled(scan_result_blob));
        }

        let db_enroll_res = match unlocked
            .facetec
            .db_enroll(ft::db_enroll::Request {
                external_database_ref_id: &public_key_hex,
                group_name: DB_GROUP_NAME,
            })
            .await
        {
            Ok(db_enroll_res) => db_enroll_res,
            Err(err) => return Err(Error::InternalErrorDbEnroll(err, scan_result_blob)),
        };

        trace!(message = "Got FaceTec 3D-DB enroll results", ?db_enroll_res);

        if !db_enroll_res.success {
            return Err(Error::InternalErrorDbEnrollUnsuccessful(scan_result_blob));
        }

        Ok(Response { scan_result_blob })
    }
}
