//! EIP-191 token claim message builder.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{vec, vec::Vec};

/// Token claim message.
pub struct Message<'a> {
    /// Substrate address.
    pub substrate_address: &'a [u8; 32],
    /// Genesis hash.
    pub genesis_hash: &'a [u8; 32],
}

impl<'a> Message<'a> {
    /// Prepare EIP-191 token claim message.
    pub fn prepare_message(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend_from_slice("I hereby sign that I claim HMND to 0x".as_bytes());
        buf.extend_from_slice(hex::encode(self.substrate_address).as_bytes());
        buf.extend_from_slice(" on network with genesis 0x".as_bytes());
        buf.extend_from_slice(hex::encode(self.genesis_hash).as_bytes());
        buf.extend_from_slice(".".as_bytes());
        buf
    }
}
