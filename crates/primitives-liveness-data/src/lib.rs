//! Plain and opaque Liveness Data.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// The data packet required to conduct liveness checks via the FaceTec Server.
#[derive(Debug, PartialEq, Encode, Decode, Serialize, Deserialize, TypeInfo)]
pub struct LivenessData {
    /// The face scan.
    pub face_scan: String,
    /// Audit trail image.
    pub audit_trail_image: String,
    /// Low quality audit trail image.
    pub low_quality_audit_trail_image: String,
}

impl TryFrom<&[u8]> for LivenessData {
    type Error = codec::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::decode(&mut &*value)
    }
}

impl From<&LivenessData> for Vec<u8> {
    fn from(value: &LivenessData) -> Vec<u8> {
        value.encode()
    }
}

/// The opaque encoded form of the [`LivenessData`].
/// Used for signing.
/// Does not guarantee that the underlying bytes indeed represent a valid [`LivenessData`] packet,
/// but allows one to attempt to decode one via [`TryFrom`].
pub type OpaqueLivenessData = Vec<u8>;

impl TryFrom<&OpaqueLivenessData> for LivenessData {
    type Error = codec::Error;

    fn try_from(value: &OpaqueLivenessData) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}
