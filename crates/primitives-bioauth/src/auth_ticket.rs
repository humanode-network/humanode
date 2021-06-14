//! Plain and opaque Auth Tickets.

use core::convert::TryFrom;

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// The one-time ticket to authenticate in the network.
#[derive(Debug, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(transparent))]
pub struct OpaqueAuthTicket(pub Vec<u8>);

/// The one-time ticket to authenticate in the network.
#[derive(Debug, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AuthTicket {
    /// The public key that matched with the provided FaceTec 3D FaceScan.
    pub public_key: Vec<u8>,
    /// Opaque one-time use value.
    /// Nonce is supposed to be unique among all of the authentication attempts,
    /// or at the very least - all authentication attempts for a particular public key.
    pub authentication_nonce: Vec<u8>,
}

impl TryFrom<&OpaqueAuthTicket> for AuthTicket {
    type Error = codec::Error;

    fn try_from(value: &OpaqueAuthTicket) -> Result<Self, Self::Error> {
        Self::decode(&mut &*value.0)
    }
}

impl From<&AuthTicket> for OpaqueAuthTicket {
    fn from(val: &AuthTicket) -> Self {
        Self(val.encode())
    }
}

impl AsRef<[u8]> for OpaqueAuthTicket {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<Vec<u8>> for OpaqueAuthTicket {
    fn from(val: Vec<u8>) -> Self {
        Self(val)
    }
}

impl From<OpaqueAuthTicket> for Vec<u8> {
    fn from(val: OpaqueAuthTicket) -> Self {
        val.0
    }
}
