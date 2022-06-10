//! Implements EIP-712 typed verification logic for the EVM account claiming.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_core_hashing_proc_macro::keccak_256 as const_keccak_256;
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};

/// EIP712Domain typehash.
const EIP712_DOMAIN_TYPEHASH: [u8; 32] = const_keccak_256!(
    b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
);
/// Account claim typehash.
const ACCOUNT_CLAIM_TYPEHASH: [u8; 32] = const_keccak_256!(b"Claim(bytes substrateAddress)");

/// A type alias representing a `string` solidity type.
type EthString = str;
/// A type alias representing a `uint256` solidity type.
type EthUint265 = [u8; 32];
/// A type alias representing an `address` solidity type.
type EthAddress = [u8; 20];
/// A type alias representing the `bytes` solidity type.
type EthBytes = [u8];

/// A first number of an EIP191 message.
const EIP191_MAGIC_BYTE: u8 = 0x19;
/// The EIP191 version for the EIP-712 structured data.
const EIP191_VERSION_STRUCTURED_DATA: u8 = 0x01;

/// Prepare a hash for the whole EIP-712 message.
fn make_eip712_message_hash(domain_separator: &[u8; 32], payload_hash: &[u8; 32]) -> [u8; 32] {
    let mut msg: [u8; 66] = [0; 66];
    msg[0] = EIP191_MAGIC_BYTE;
    msg[1] = EIP191_VERSION_STRUCTURED_DATA;
    msg[2..34].copy_from_slice(domain_separator);
    msg[34..66].copy_from_slice(payload_hash);
    keccak_256(&msg)
}

/// The EIP712 domain.
pub struct Domain<'a> {
    /// The name of the domain.
    pub name: &'a EthString,
    /// The version of the domain.
    /// Bump this value if you need to make the old signed messages obsolete.
    pub version: &'a EthString,
    /// The Chain ID of the Ethereum chain this code runs at.
    pub chain_id: &'a EthUint265,
    /// The verifying contract, indeteded for the address of the contract that will be verifying
    /// the signature.
    pub verifying_contract: &'a EthAddress,
}

/// Prepare a hash for EIP712Domain data type.
fn make_domain_hash(domain: Domain<'_>) -> [u8; 32] {
    let mut buf = [0u8; 148];
    buf[0..32].copy_from_slice(&EIP712_DOMAIN_TYPEHASH);
    buf[32..64].copy_from_slice(&keccak_256(domain.name.as_bytes()));
    buf[64..96].copy_from_slice(&keccak_256(domain.version.as_bytes()));
    buf[96..128].copy_from_slice(domain.chain_id);
    buf[128..148].copy_from_slice(domain.verifying_contract);
    keccak_256(&buf)
}

/// Prepare a hash for our account claim data type.
/// To be used at EIP-712 message payload.
fn make_account_claim_hash(account: &EthBytes) -> [u8; 32] {
    let mut buf = [0u8; 64];
    buf[0..32].copy_from_slice(&ACCOUNT_CLAIM_TYPEHASH);
    buf[32..64].copy_from_slice(&keccak_256(account));
    keccak_256(&buf)
}

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Signature = [u8; 65];

/// Verify EIP-712 typed signature based on provided domain_separator and entire message.
pub fn verify_account_claim(
    domain: Domain<'_>,
    account: &[u8],
    signature: Signature,
) -> Option<[u8; 20]> {
    let domain_hash = make_domain_hash(domain);
    let account_claim_hash = make_account_claim_hash(account);
    let msg_hash = make_eip712_message_hash(&domain_hash, &account_claim_hash);
    recover_signer(&signature, &msg_hash)
}

/// Convert the EVM public key to an EVM address.
fn evm_address_from_evm_pubkey(pubkey: &[u8; 64]) -> [u8; 20] {
    let mut buf = [0u8; 20];
    buf.copy_from_slice(&keccak_256(pubkey)[12..]);
    buf
}

/// Extract the signer address from the signatue and the message.
fn recover_signer(sig: &Signature, msg: &[u8; 32]) -> Option<[u8; 20]> {
    secp256k1_ecdsa_recover(sig, msg)
        .map(|pubkey| evm_address_from_evm_pubkey(&pubkey))
        .ok()
}

#[cfg(test)]
mod tests {
    use eth_eip_712::{hash_structured_data, EIP712};
    use hex_literal::hex;
    use serde_json::from_str;
    use sp_core::U256;

    use super::*;

    // Alice secret key.
    fn alice_secret() -> libsecp256k1::SecretKey {
        libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
    }

    // Alice public key.
    fn alice_public() -> [u8; 20] {
        let full_pubkey = libsecp256k1::PublicKey::from_secret_key(&alice_secret()).serialize();
        let mut pubkey = [0u8; 64];
        pubkey.copy_from_slice(&full_pubkey[1..65]);
        evm_address_from_evm_pubkey(&pubkey)
    }

    // A helper function to sign a message.
    fn eth_sign(secret: &libsecp256k1::SecretKey, msg: [u8; 32]) -> Signature {
        let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
        let mut r = [0u8; 65];
        r[0..64].copy_from_slice(&sig.serialize()[..]);
        r[64] = recovery_id.serialize();
        r
    }

    // A helper function to construct test EIP-712 signature.
    fn alice_test_input() -> Signature {
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
                    { "name": "verifyingContract", "type": "address" }
                ],
                "Claim": [
                    { "name": "substrateAddress", "type": "bytes" }
                ]
            }
        }"#;
        let typed_data = from_str::<EIP712>(claim_eip_712_json).unwrap();
        let msg_bytes: [u8; 32] = hash_structured_data(typed_data).unwrap().into();

        let secret = alice_secret();
        eth_sign(&secret, msg_bytes)
    }

    // A helper function to prepare alice account claim typed data.
    fn prepare_sample_domain() -> Domain<'static> {
        Domain {
            name: "Humanode EVM Claim",
            version: "1",
            chain_id: &hex!("0000000000000000000000000000000000000000000000000000000000000001"),
            verifying_contract: &hex!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"),
        }
    }

    #[test]
    fn verify_test_chain_id() {
        let domain = prepare_sample_domain();
        let expected_chain_id: [u8; 32] = U256::from("0x1").into();
        assert_eq!(domain.chain_id, &expected_chain_id);
    }

    #[test]
    fn verify_domain_separator() {
        // See https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol

        // From https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol#L101
        let sample_separator: [u8; 32] =
            U256::from("0xf2cee375fa42b42143804025fc449deafd50cc031ca257e0b194a650a912090f").into();

        // Sample test data
        // See https://github.com/ethereum/EIPs/blob/fcaec3dc70e758fe80abd86f0c70bbbedbec6e61/assets/eip-712/Example.sol#L38-L44
        let verifying_contract: [u8; 20] = hex!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC");
        let chain_id: [u8; 32] = U256::from("0x1").into();
        let domain = Domain {
            name: "Ether Mail",
            version: "1",
            chain_id: &chain_id,
            verifying_contract: &verifying_contract,
        };

        assert_eq!(make_domain_hash(domain), sample_separator);
    }

    const SAMPLE_ACCOUNT: [u8; 32] =
        hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");

    #[test]
    fn valid_signature() {
        let signature = alice_test_input();
        let domain = prepare_sample_domain();

        let evm_address = verify_account_claim(domain, &SAMPLE_ACCOUNT, signature).unwrap();
        assert_eq!(evm_address, alice_public());
    }

    #[test]
    fn invalid_signature() {
        let signature = [1u8; 65];
        let domain = prepare_sample_domain();

        let evm_address = verify_account_claim(domain, &SAMPLE_ACCOUNT, signature).unwrap();
        assert_ne!(evm_address, alice_public());
    }
}
