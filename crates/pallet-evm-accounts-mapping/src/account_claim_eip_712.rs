//! Implements EIP-712 typed verification logic for evm account claiming.

use module_evm_utility_macro::keccak256;
use sp_core::{H256, U256};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_std::prelude::*;

use crate::EvmAddress;

/// Domain name.
pub const NAME: &str = "Humanode EVM Claim";
/// Domain version.
pub const VERSION: &str = "1";

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Signature = [u8; 65];

/// TODO!
pub struct AccountClaimTypedData {
    /// TODO!
    pub name: &'static str,
    /// TODO!
    pub version: &'static str,
    /// TODO!
    pub chain_id: u64,
    /// TODO!
    pub genesis_block_hash: Vec<u8>,
    /// TODO!
    pub account: Vec<u8>,
}

/// TODO!
pub fn to_bytes<T: Into<U256>>(value: T) -> [u8; 32] {
    Into::<[u8; 32]>::into(value.into())
}

impl AccountClaimTypedData {
    /// TODO!
    fn evm_account_payload_hash(&self) -> [u8; 32] {
        let tx_type_hash = keccak256!("Claim(bytes substrateAddress)");
        let mut tx_msg = tx_type_hash.to_vec();
        tx_msg.extend_from_slice(&keccak_256(&self.account));
        keccak_256(tx_msg.as_slice())
    }

    /// TODO!
    fn eip712_signable_message(&self) -> Vec<u8> {
        let domain_separator = self.evm_account_domain_separator();
        let payload_hash = self.evm_account_payload_hash();

        let mut msg = b"\x19\x01".to_vec();
        msg.extend_from_slice(&domain_separator);
        msg.extend_from_slice(&payload_hash);
        msg
    }

    /// TODO!
    fn evm_account_domain_separator(&self) -> [u8; 32] {
        let domain_hash = keccak256!(
            "EIP712Domain(string name,string version,uint256 chainId,bytes verifyingContract)"
        );
        let mut domain_seperator_msg = domain_hash.to_vec();
        domain_seperator_msg.extend_from_slice(keccak256!("Humanode EVM Claim"));
        domain_seperator_msg.extend_from_slice(keccak256!("1"));
        domain_seperator_msg.extend_from_slice(&to_bytes(1));
        domain_seperator_msg.extend_from_slice(&keccak_256(&self.genesis_block_hash));
        keccak_256(domain_seperator_msg.as_slice())
    }
}

/// Provides the capability to verify an EIP-712 based ethereum signature.
pub trait Verifier {
    /// Verify the signature and extract a corresponding [`EvmAddress`] if it's ok.
    fn verify(
        account_claimed_typed_data: AccountClaimTypedData,
        signature: Signature,
    ) -> Option<EvmAddress>;
}

/// Verify EIP-712 typed signature based on provided domain_separator and entire message.
pub struct VerifierFactory;

impl Verifier for VerifierFactory {
    fn verify(
        account_claimed_typed_data: AccountClaimTypedData,
        signature: Signature,
    ) -> Option<EvmAddress> {
        let msg = account_claimed_typed_data.eip712_signable_message();
        let msg_hash = keccak_256(msg.as_slice());

        recover_signer(&signature, &msg_hash)
    }
}

/// A helper function to return a corresponding [`EvmAddress`] from signature and message hash.
fn recover_signer(sig: &Signature, msg_hash: &[u8; 32]) -> Option<EvmAddress> {
    secp256k1_ecdsa_recover(sig, msg_hash)
        .map(|pubkey| EvmAddress::from(H256::from_slice(&keccak_256(&pubkey))))
        .ok()
}

#[cfg(test)]
mod tests {

    use eip_712::{hash_structured_data, EIP712};
    use serde_json::from_str;

    use super::*;

    // A helper function to construct a message and sign it.
    fn eth_sign(secret: &libsecp256k1::SecretKey, msg: [u8; 32]) -> Signature {
        let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
        let mut r = [0u8; 65];
        r[0..64].copy_from_slice(&sig.serialize()[..]);
        r[64] = recovery_id.serialize();
        r
    }

    fn alice_test_input() -> (Signature, EvmAddress) {
        let claim_eip_712_json = r#"{
            "primaryType": "Claim",
            "domain": {
                "name": "Humanode EVM Claim",
                "version": "1",
                "chainId": "0x1",
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "substrateAddress": "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
            },
            "types": {
                "EIP712Domain": [
                    { "name": "name", "type": "string" },
                    { "name": "version", "type": "string" },
                    { "name": "chainId", "type": "uint256" },
                    { "name": "verifyingContract", "type": "bytes" }
                ],
                "Claim": [
                    { "name": "substrateAddress", "type": "bytes" }
                ]
            }
        }"#;
        let typed_data = from_str::<EIP712>(claim_eip_712_json).unwrap();
        let msg_bytes: [u8; 32] = hash_structured_data(typed_data).unwrap().into();

        let secret = libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap();
        let signature = eth_sign(&secret, msg_bytes);

        let evm_address = EvmAddress::from_slice(
            &keccak_256(&libsecp256k1::PublicKey::from_secret_key(&secret).serialize()[1..65])
                [12..],
        );

        (signature, evm_address)
    }

    #[test]
    fn valid_claim() {
        let (signature, expected_evm_address) = alice_test_input();
        let alice_claim = AccountClaimTypedData {
            name: "Humanode EVM Claim",
            version: "1",
            chain_id: 1,
            genesis_block_hash: hex::decode("cccccccccccccccccccccccccccccccccccccccc").unwrap(),
            account: hex::decode(
                "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
            )
            .unwrap(),
        };

        let evm_address = VerifierFactory::verify(alice_claim, signature).unwrap();
        assert_eq!(evm_address, expected_evm_address);
    }
}
