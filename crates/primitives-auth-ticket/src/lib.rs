//! Plain and opaque Auth Tickets.

// Either generate code at stadard mode, or `no_std`, based on the `std` feature presence.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// The one-time ticket to authenticate in the network.
pub type OpaqueAuthTicket = Vec<u8>;

/// The one-time ticket to authenticate in the network.
#[derive(Debug, PartialEq, Encode, Decode, TypeInfo)]
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
        Self::decode(&mut &**value)
    }
}

impl From<&AuthTicket> for OpaqueAuthTicket {
    fn from(val: &AuthTicket) -> Self {
        val.encode()
    }
}
