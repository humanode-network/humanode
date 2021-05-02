//! Core logic of the system.

use tokio::sync::Mutex;

use crate::sequence::Sequence;
use serde::{Deserialize, Serialize};

/// The inner state, to be hidden behind the mutex to ensure we don't have
/// access to it unless we lock the mutex.
pub struct Locked {
    /// The sequence number.
    pub sequence: Sequence,
    /// The client for the FaceTec Server.
    pub facetec: (),
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
    /// This public key is already used.
    AlreadyEnrolled,
}

impl Logic {
    /// An enroll invocation handler.
    pub async fn enroll(&self, req: EnrollRequest) -> Result<(), EnrollError> {
        let mut _unlocked = self.locked.lock().await;
        // unlocked.facetec.enrollment_3d(&req.public_key, &req.face_scan).await?;
        // match unlocked.facetec.3d_db_search(&req.public_key).await {
        //     Err(NotFound) => {},
        //     Ok(_) => return Ok(Response::builder().status(409).body(Body::empty())?),
        //     Err(error) => return Ok(Response::builder().status(500).body(Body::new(error))?),
        // }
        // unlocked.facetec.3d_db_enroll(&public_key).await?;
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
    /// The FaceScan did not match.
    NotFound,
}

impl Logic {
    /// An authenticate invocation handler.
    pub async fn authenticate(
        &self,
        req: AuthenticateRequest,
    ) -> Result<AuthenticateResponse, AuthenticateError> {
        let mut unlocked = self.locked.lock().await;
        unlocked.sequence.inc();
        // unlocked.facetec.enroll(unlocked.sequence.get(), face_scan).await;
        // let public_key = unlocked.facetec.3d_db_search(unlocked.sequence.get()).await?;
        // public_key.validate(face_scan_signature)?;
        // let signed_public_key = unlocked.signer.sign(public_key);
        // return both public_key and signed_public_key
        Ok(AuthenticateResponse {
            public_key: String::new(),
            authentication_signature: String::new(),
        })
    }
}
