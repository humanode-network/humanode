//! Implements EIP-712 typed verification logic for the token claiming.

#![cfg_attr(not(feature = "std"), no_std)]

use eip712_common::{
    const_keccak_256, keccak_256, make_payload_hash, Domain, EcdsaSignature, EthBytes,
    EthereumAddress,
};

/// Token claim typehash.
const TOKEN_CLAIM_TYPEHASH: [u8; 32] = const_keccak_256!(b"TokenClaim(bytes substrateAddress)");

/// Make the data hash from the `TokenClaim` payload.
fn hash_token_claim_data(account: &EthBytes) -> [u8; 32] {
    keccak_256(account)
}

/// Verify EIP-712 typed signature based on provided domain_separator and entire message.
pub fn verify_token_claim(
    signature: &EcdsaSignature,
    domain: Domain<'_>,
    account: &[u8],
) -> Option<EthereumAddress> {
    let payload_hash = make_payload_hash(&TOKEN_CLAIM_TYPEHASH, [&hash_token_claim_data(account)]);
    eip712_common::verify_signature(signature, domain, &payload_hash)
}

#[cfg(test)]
mod tests {
    use eip712_common_test_utils::{
        ecdsa, ecdsa_pair, ecdsa_sign_typed_data, ethereum_address_from_seed,
    };
    use hex_literal::hex;

    use super::*;

    fn test_input(pair: &ecdsa::Pair) -> EcdsaSignature {
        let type_data_json = r#"{
            "primaryType": "TokenClaim",
            "domain": {
                "name": "Humanode Token Claim",
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
                "TokenClaim": [
                    { "name": "substrateAddress", "type": "bytes" }
                ]
            }
        }"#;
        ecdsa_sign_typed_data(pair, type_data_json)
    }

    fn prepare_sample_domain() -> Domain<'static> {
        Domain {
            name: "Humanode Token Claim",
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

        let ethereum_address = verify_token_claim(&signature, domain, &SAMPLE_ACCOUNT).unwrap();
        assert_eq!(ethereum_address, ethereum_address_from_seed(b"Alice"));
    }

    #[test]
    fn invalid_signature() {
        let pair = ecdsa_pair(b"Alice");
        let signature = test_input(&pair);
        let domain = prepare_sample_domain();

        let ethereum_address = verify_token_claim(&signature, domain, &SAMPLE_ACCOUNT).unwrap();
        assert_ne!(ethereum_address, ethereum_address_from_seed(b"Bob"));
    }
}
