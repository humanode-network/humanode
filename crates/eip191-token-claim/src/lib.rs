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

impl Message<'_> {
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

#[cfg(test)]
mod tests {

    use hex_literal::hex;
    use primitives_ethereum::EcdsaSignature;

    use super::*;

    /// This test contains the data obtained from MetaMask browser extension via an injected web3
    /// interface.
    /// It validates that the real-world external ecosystem works properly with our code.
    #[test]
    fn real_world_case() {
        let substrate_address =
            hex!("1e38cdd099576380ca4df726fa8b740d3ae6b159e71cd5ef7aa621f5bd01d653");
        let genesis_hash = hex!("bed15072ffa35432da5d20c33920b3afc2ab850e864a26b684e7f6caed6a1479");

        let token_claim_message = Message {
            substrate_address: &substrate_address,
            genesis_hash: &genesis_hash,
        };

        let signature = hex!("f76b13746bb661fb6a1242b5591d4442a88a09c1600c5ccb77e7083f37b7d17e6b975bbdd88e186870fcadd464dcff0a4b4f6d32e4a51291d4b1f543ea588ae11c");

        let ethereum_address = eip191_crypto::recover_signer(
            &EcdsaSignature(signature),
            token_claim_message.prepare_message().as_slice(),
        )
        .unwrap();

        assert_eq!(
            ethereum_address.0,
            hex!("f24ff3a9cf04c71dbc94d0b566f7a27b94566cac"),
        );
    }
}
