//! Implements EIP-712 typed verification logic for evm account claiming.

use sha3::Digest;
use sp_core::{H256, U256};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_std::prelude::*;

use crate::EvmAddress;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Signature = [u8; 65];

struct AccountClaim {}

/// A first number of an EIP191 message.
const EIP191_MAGIC_BYTE: u8 = 0x19;
/// The EIP191 version for the EIP-712 structured data.
const EIP191_STRUCTURED_DATA: u8 = 0x01;

// /// TODO!
// fn evm_account_payload_hash(&self) -> [u8; 32] {
//     let tx_type_hash = keccak256!("Claim(bytes substrateAddress)");
//     let mut tx_msg = tx_type_hash.to_vec();
//     tx_msg.extend_from_slice(&keccak_256(&self.account));
//     keccak_256(tx_msg.as_slice())
// }

// /// TODO!
// fn eip712_signable_message(&self) -> Vec<u8> {
//     let domain_separator = self.evm_account_domain_separator();
//     let payload_hash = self.evm_account_payload_hash();

//     let mut msg = b"\x19\x01".to_vec();
//     msg.extend_from_slice(&domain_separator);
//     msg.extend_from_slice(&payload_hash);
//     msg
// }

/// Verify EIP-712 typed signature based on provided domain_separator and entire message.
pub struct Verifier;

impl Verifier {
    pub fn verify(account_claimed_typed_data: (), signature: Signature) -> Option<EvmAddress> {
        // let msg = account_claimed_typed_data.eip712_signable_message();
        // let msg_hash = keccak_256(msg.as_slice());
        let msg_hash = [0u8; 32];

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

    // // A helper function to construct a message and sign it.
    // fn eth_sign(secret: &libsecp256k1::SecretKey, msg: [u8; 32]) -> Signature {
    //     let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
    //     let mut r = [0u8; 65];
    //     r[0..64].copy_from_slice(&sig.serialize()[..]);
    //     r[64] = recovery_id.serialize();
    //     r
    // }

    // fn alice_test_input() -> (Signature, EvmAddress) {
    //     let claim_eip_712_json = r#"{
    //         "primaryType": "Claim",
    //         "domain": {
    //             "name": "Humanode EVM Claim",
    //             "version": "1",
    //             "chainId": "0x1",
    //             "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
    //         },
    //         "message": {
    //             "substrateAddress": "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
    //         },
    //         "types": {
    //             "EIP712Domain": [
    //                 { "name": "name", "type": "string" },
    //                 { "name": "version", "type": "string" },
    //                 { "name": "chainId", "type": "uint256" },
    //                 { "name": "verifyingContract", "type": "bytes" }
    //             ],
    //             "Claim": [
    //                 { "name": "substrateAddress", "type": "bytes" }
    //             ]
    //         }
    //     }"#;
    //     let typed_data = from_str::<EIP712>(claim_eip_712_json).unwrap();
    //     let msg_bytes: [u8; 32] = hash_structured_data(typed_data).unwrap().into();

    //     let secret = libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap();
    //     let signature = eth_sign(&secret, msg_bytes);

    //     let evm_address = EvmAddress::from_slice(
    //         &keccak_256(&libsecp256k1::PublicKey::from_secret_key(&secret).serialize()[1..65])
    //             [12..],
    //     );

    //     (signature, evm_address)
    // }

    // #[test]
    // fn valid_claim() {
    //     let (signature, expected_evm_address) = alice_test_input();
    //     let alice_claim = TypedDataV4 {
    //         name: "Humanode EVM Claim",
    //         version: "1",
    //         chain_id: 1,
    //         genesis_block_hash: hex::decode("cccccccccccccccccccccccccccccccccccccccc").unwrap(),
    //         account: hex::decode(
    //             "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
    //         )
    //         .unwrap(),
    //     };

    //     let evm_address = VerifierFactory::verify(alice_claim, signature).unwrap();
    //     assert_eq!(evm_address, expected_evm_address);
    // }

    #[test]
    fn domain_typehash_full() {
        let sample_hash = keccak_256(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)");

        let domain = EIP712Domain {
            name: Some(""),
            version: Some(""),
            chain_id: Some(&[0u8; 32]),
            verifying_contract: Some(&[0u8; 32]),
            salt: Some(&[0u8; 32]),
        };
        let computed_hash = domain.typehash();

        assert_eq!(sample_hash, computed_hash)
    }

    #[test]
    fn domain_separator() {
        // See https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol

        // From https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol#L101
        let sample_separator: [u8; 32] =
            U256::from("0xf2cee375fa42b42143804025fc449deafd50cc031ca257e0b194a650a912090f").into();

        // Sample test data
        // See https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol#L38-L44
        let verifying_contract: [u8; 32] =
            U256::from("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC").into();
        let chain_id: [u8; 32] = U256::from(1).into();
        let domain = EIP712Domain {
            name: Some("Ether Mail"),
            version: Some("1"),
            chain_id: Some(&chain_id),
            verifying_contract: Some(&verifying_contract),
            salt: None,
        };

        assert_eq!(domain.domain_separator(), sample_separator);
    }
}
