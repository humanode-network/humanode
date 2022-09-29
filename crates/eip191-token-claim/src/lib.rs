//! E.

#![cfg_attr(not(feature = "std"), no_std)]

pub use sp_core_hashing_proc_macro::keccak_256 as const_keccak_256;
pub use sp_io::hashing::keccak_256;

/// Token claim message.
pub struct Message<'a> {
    /// Substrate address.
    pub substrate_address: &'a [u8; 32],
    /// Genesis hash.
    pub genesis_hash: &'a [u8; 32],
}

impl<'a> Message<'a> {
    /// Prepare token claim message.
    fn prepare_message(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend_from_slice("I hereby sign that I claim HMND to ".as_bytes());
        buf.extend_from_slice(self.substrate_address);
        buf.extend_from_slice(" on network with genesis ".as_bytes());
        buf.extend_from_slice(self.genesis_hash);
        buf
    }
}
