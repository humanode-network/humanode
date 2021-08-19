//! Enroll operation.

use std::convert::TryFrom;

use facetec_api_client as ft;
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use serde::Deserialize;
use tracing::{error, trace};

use crate::logic::facetec_utils::{db_search_result_adapter, DbSearchResult};

use super::{common::*, Logic, Signer};

/// The request for the enroll operation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// The public key of the validator.
    pub public_key: Vec<u8>,
    /// The liveness data that the validator owner provided.
    pub liveness_data: OpaqueLivenessData,
}

/// The errors on the enroll operation.
#[derive(Debug)]
pub enum Error {
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

#[async_trait::async_trait]
impl<S, PK> crate::http::traits::Enroll for Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]> + AsRef<[u8]>,
{
    /// An enroll invocation handler.
    async fn enroll(&self, req: Request) -> Result<(), Error> {
        let public_key = PK::try_from(&req.public_key).map_err(|_| Error::InvalidPublicKey)?;

        let liveness_data =
            LivenessData::try_from(&req.liveness_data).map_err(Error::InvalidLivenessData)?;

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
            .map_err(Error::InternalErrorEnrollment)?;

        trace!(message = "Got FaceTec enroll results", ?enroll_res);

        if !enroll_res.success {
            error!(
                message = "Unsuccessful enroll response from FaceTec server during robonode enroll",
                ?enroll_res
            );
            if let Some(error_message) = enroll_res.error_message {
                if error_message == EXTERNAL_DATABASE_REF_ID_ALREADY_IN_USE_ERROR_MESSAGE {
                    return Err(Error::PublicKeyAlreadyUsed);
                }
            } else if let Some(face_scan) = enroll_res.face_scan {
                if !face_scan.face_scan_security_checks.all_checks_succeeded() {
                    return Err(Error::FaceScanRejected);
                }
            }
            return Err(Error::InternalErrorEnrollmentUnsuccessful);
        }

        drop(enroll_res);

        let search_result = unlocked
            .facetec
            .db_search(ft::db_search::Request {
                external_database_ref_id: &public_key_hex,
                group_name: DB_GROUP_NAME,
                min_match_level: MATCH_LEVEL,
            })
            .await;

        let results = match db_search_result_adapter(search_result) {
            DbSearchResult::OtherError(err) => return Err(Error::InternalErrorDbSearch(err)),
            DbSearchResult::NoGroupError => {
                trace!(message = "Got no-group error instead of FaceTec 3D-DB search results, assuming no results");
                vec![]
            }
            DbSearchResult::Response(search_res) => {
                trace!(message = "Got FaceTec 3D-DB search results", ?search_res);
                if !search_res.success {
                    return Err(Error::InternalErrorDbSearchUnsuccessful);
                }
                search_res.results
            }
        };

        // If the results set is non-empty - this means that this person has
        // already enrolled with the system. It might also be a false-positive.
        if !results.is_empty() {
            return Err(Error::PersonAlreadyEnrolled);
        }

        let db_enroll_res = unlocked
            .facetec
            .db_enroll(ft::db_enroll::Request {
                external_database_ref_id: &public_key_hex,
                group_name: DB_GROUP_NAME,
            })
            .await
            .map_err(Error::InternalErrorDbEnroll)?;

        trace!(message = "Got FaceTec 3D-DB enroll results", ?db_enroll_res);

        if !db_enroll_res.success {
            return Err(Error::InternalErrorDbEnrollUnsuccessful);
        }

        drop(db_enroll_res);

        Ok(())
    }
}
