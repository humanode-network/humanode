//! Implements EIP-712 typed verification logic.

use sp_core::H256;
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_std::prelude::*;

use crate::EvmAddress;

/// Domain separator type definition.
pub const DOMAIN: &str = "EIP712Domain(string name,string version,uint256 chainId,bytes32 salt)";
/// Domain name.
pub const NAME: &str = "Humanode EVM Claim";
/// Domain version.
pub const VERSION: u8 = 1;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Signature = [u8; 65];

/// Provides the capability to verify an EIP-712 based ethereum signature.
pub trait Verifier {
    /// Verify the signature and extract a corresponding [`EvmAddress`] if it's ok.
    fn verify(
        domain_separator: [u8; 32],
        message: Vec<u8>,
        signature: Signature,
    ) -> Option<EvmAddress>;
}

/// Verify EIP-712 typed signature based on provided domain_separator and entire message.
pub struct VerifierFactory;

impl Verifier for VerifierFactory {
    fn verify(
        domain_separator: [u8; 32],
        message: Vec<u8>,
        signature: Signature,
    ) -> Option<EvmAddress> {
        let msg = Self::eip712_signable_message(domain_separator, message);
        let msg_hash = keccak_256(msg.as_slice());

        recover_signer(&signature, &msg_hash)
    }
}

impl VerifierFactory {
    /// EIP-712 message to be signed.
    fn eip712_signable_message(domain_separator: [u8; 32], message: Vec<u8>) -> Vec<u8> {
        let payload_hash = Self::payload_hash(message);

        let mut msg = b"\x19\x01".to_vec();
        msg.extend_from_slice(&domain_separator);
        msg.extend_from_slice(&payload_hash);
        msg
    }

    /// Get payload hash from message.
    fn payload_hash(message: Vec<u8>) -> [u8; 32] {
        keccak_256(message.as_slice())
    }
}

/// A helper function to return a corresponding [`EvmAddress`] from signature and message hash.
fn recover_signer(sig: &Signature, msg_hash: &[u8; 32]) -> Option<EvmAddress> {
    secp256k1_ecdsa_recover(sig, msg_hash)
        .map(|pubkey| EvmAddress::from(H256::from_slice(&keccak_256(&pubkey))))
        .ok()
}
