//! Implements EIP-712 typed verification logic for the EVM account claiming.

#![cfg_attr(not(feature = "std"), no_std)]

use eip712_common::{
    const_keccak_256, keccak_256, Domain, EcdsaSignature, EthBytes, EthereumAddress,
};

/// Account claim typehash.
const ACCOUNT_CLAIM_TYPEHASH: [u8; 32] = const_keccak_256!(b"Claim(bytes substrateAddress)");

/// Prepare a hash for our account claim data type.
/// To be used at EIP-712 message payload.
fn make_account_claim_hash(account: &EthBytes) -> [u8; 32] {
    let mut buf = [0u8; 64];
    buf[0..32].copy_from_slice(&ACCOUNT_CLAIM_TYPEHASH);
    buf[32..64].copy_from_slice(&keccak_256(account));
    keccak_256(&buf)
}

/// Verify EIP-712 typed signature based on provided domain_separator and entire message.
pub fn verify_account_claim(
    signature: &EcdsaSignature,
    domain: Domain<'_>,
    account: &[u8],
) -> Option<EthereumAddress> {
    let payload_hash = make_account_claim_hash(account);
    eip712_common::verify_signature(signature, domain, &payload_hash)
}

#[cfg(test)]
mod tests {
    use eip712_common_test_utils::{
        ecdsa, ecdsa_pair, ecdsa_sign_typed_data, ethereum_address_from_seed, U256,
    };
    use hex_literal::hex;

    use super::*;

    // A helper function to construct test EIP-712 signature.
    fn test_input(pair: &ecdsa::Pair) -> EcdsaSignature {
        let typed_data_json = r#"{
            "primaryType": "Claim",
            "domain": {
                "name": "Humanode EVM Claim",
                "version": "1",
                "chainId": "0x1472",
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
        ecdsa_sign_typed_data(pair, typed_data_json)
    }

    // A helper function to prepare alice account claim typed data.
    fn prepare_sample_domain() -> Domain<'static> {
        Domain {
            name: "Humanode EVM Claim",
            version: "1",
            // Chain ID is 5234 in hex.
            chain_id: &hex!("0000000000000000000000000000000000000000000000000000000000001472"),
            verifying_contract: &hex!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"),
        }
    }

    const SAMPLE_ACCOUNT: [u8; 32] =
        hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");

    #[test]
    fn valid_signature() {
        let pair = ecdsa_pair(b"Alice");
        let signature = test_input(&pair);
        let domain = prepare_sample_domain();

        let ethereum_address = verify_account_claim(&signature, domain, &SAMPLE_ACCOUNT).unwrap();
        assert_eq!(ethereum_address, ethereum_address_from_seed(b"Alice"));
    }

    #[test]
    fn invalid_signature() {
        let pair = ecdsa_pair(b"Alice");
        let signature = test_input(&pair);
        let domain = prepare_sample_domain();

        let ethereum_address = verify_account_claim(&signature, domain, &SAMPLE_ACCOUNT).unwrap();
        assert_ne!(ethereum_address, ethereum_address_from_seed(b"Bob"));
    }

    #[test]
    fn real_world_case1() {
        let chain_id: [u8; 32] = U256::from(5234).into();
        let domain = Domain {
            name: "Humanode EVM Claim",
            version: "1",
            chain_id: &chain_id,
            verifying_contract: &hex!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"),
        };
        let account_to_claim =
            hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
        let signature = hex!("151d5f52e6c249db84b8705374c6f51dd08b50ddad5b1175ec20a7e00cbc48f55a23470ab0db16146b3b7d2a8565aaf2b700f548c9e9882a0034e654bd214e821b");

        let ethereum_address =
            verify_account_claim(&EcdsaSignature(signature), domain, &account_to_claim).unwrap();
        assert_eq!(
            ethereum_address.0,
            hex!("e9726f3d0a7736034e2a4c63ea28b3ab95622cb9"),
        );
    }
}
