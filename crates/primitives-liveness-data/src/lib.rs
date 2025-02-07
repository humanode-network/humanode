//! Plain and opaque Liveness Data.

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// The data packet required to conduct liveness checks via the FaceTec Server.
#[derive(Debug, PartialEq, Encode, Decode, Serialize, Deserialize, TypeInfo)]
#[serde(rename_all = "camelCase")]
pub struct LivenessData {
    /// The face scan.
    pub face_scan: String,
    /// Audit trail image.
    pub audit_trail_image: String,
    /// Low quality audit trail image.
    pub low_quality_audit_trail_image: String,
}

/// The opaque encoded form of the [`LivenessData`].
/// Used for signing.
/// Does not guarantee that the underlying bytes indeed represent a valid [`LivenessData`] packet,
/// but allows one to attempt to decode one via [`TryFrom`].
#[derive(Debug, PartialEq, Encode, Decode, Serialize, Deserialize, TypeInfo)]
#[serde(transparent)]
pub struct OpaqueLivenessData(pub Vec<u8>);

impl From<Vec<u8>> for OpaqueLivenessData {
    fn from(val: Vec<u8>) -> Self {
        Self(val)
    }
}

impl AsRef<[u8]> for OpaqueLivenessData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl TryFrom<&OpaqueLivenessData> for LivenessData {
    type Error = codec::Error;

    fn try_from(value: &OpaqueLivenessData) -> Result<Self, Self::Error> {
        Self::decode(&mut &*value.0)
    }
}

impl From<&LivenessData> for OpaqueLivenessData {
    fn from(val: &LivenessData) -> Self {
        Self(val.encode())
    }
}

/// A reference to an opaque encoded form of the [`LivenessData`].
/// Does not guarantee that the underlying bytes indeed represent a valid [`LivenessData`] packet,
/// but allows one to attempt to decode one via [`TryFrom`].
/// For use at encoding and serialization to avoid data copies.
#[derive(Debug, PartialEq, Encode, Serialize)]
#[serde(transparent)]
pub struct OpaqueLivenessDataRef<'a>(pub &'a [u8]);

impl<'a> From<&'a [u8]> for OpaqueLivenessDataRef<'a> {
    fn from(val: &'a [u8]) -> Self {
        Self(val)
    }
}

impl AsRef<[u8]> for OpaqueLivenessDataRef<'_> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> TryFrom<OpaqueLivenessDataRef<'a>> for LivenessData {
    type Error = codec::Error;

    fn try_from(value: OpaqueLivenessDataRef<'a>) -> Result<Self, Self::Error> {
        Self::decode(&mut &*value.0)
    }
}
