//! Implements EIP-712 typed verification logic for the token claiming.

#![cfg_attr(not(feature = "std"), no_std)]

use eip712_common::{
    const_keccak_256, keccak_256, make_payload_hash, Domain, EcdsaSignature, EthBytes,
    EthereumAddress,
};

/// Token claim typehash.
const TOKEN_CLAIM_TYPEHASH: [u8; 32] = const_keccak_256!(b"TokenClaim(bytes substrateAddress)");

/// Make the data hash from the `TokenClaim` payload.
fn hash_token_claim_data(substrate_address: &EthBytes) -> [u8; 32] {
    keccak_256(substrate_address)
}

/// Prepare the EIP-712 message hash.
pub fn make_message_hash(domain: Domain<'_>, substrate_address: &[u8]) -> [u8; 32] {
    let payload_hash = make_payload_hash(
        &TOKEN_CLAIM_TYPEHASH,
        [&hash_token_claim_data(substrate_address)],
    );
    eip712_common::make_message_hash(domain, &payload_hash)
}

/// Verify EIP-712 typed signature based on provided domain and message params and recover
/// the signer address.
pub fn recover_signer(
    signature: &EcdsaSignature,
    domain: Domain<'_>,
    substrate_address: &[u8],
) -> Option<EthereumAddress> {
    let message = make_message_hash(domain, substrate_address);
    eip712_common::recover_signer(signature, &message)
}

#[cfg(test)]
mod tests {
    use eip712_common_test_utils::{
        ecdsa, ecdsa_pair, ecdsa_sign_typed_data, ethereum_address_from_seed, U256,
    };
    use hex_literal::hex;

    use super::*;

    fn test_input(pair: &ecdsa::Pair) -> EcdsaSignature {
        let type_data_json = r#"{
            "primaryType": "TokenClaim",
            "domain": {
                "name": "Humanode Token Claim",
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
            // Chain ID is 1 in hex.
            chain_id: &hex!("0000000000000000000000000000000000000000000000000000000000000001"),
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

        let ethereum_address = recover_signer(&signature, domain, &SAMPLE_ACCOUNT).unwrap();
        assert_eq!(ethereum_address, ethereum_address_from_seed(b"Alice"));
    }

    #[test]
    fn invalid_signature() {
        let pair = ecdsa_pair(b"Alice");
        let signature = test_input(&pair);
        let domain = prepare_sample_domain();

        let ethereum_address = recover_signer(&signature, domain, &SAMPLE_ACCOUNT).unwrap();
        assert_ne!(ethereum_address, ethereum_address_from_seed(b"Bob"));
    }

    /// This test contains the data obtained from MetaMask browser extension via an injected web3
    /// interface.
    /// It validates that the real-world external ecosystem works properly with our code.
    #[test]
    fn real_world_case1() {
        let chain_id: [u8; 32] = U256::from(1).into();
        let domain = Domain {
            name: "Humanode Token Claim",
            version: "1",
            chain_id: &chain_id,
            verifying_contract: &hex!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"),
        };
        let substrate_account =
            hex!("181bafa36430dfc3ff5e51e34ce78dcda0dc6b6ded9b9c7d6c22f971738c9d6f");
        let signature = hex!("3027e569de1d835350ffa4f07218d3be7298de65f12ffc767c6d80ab16ee704245e158f660817433f3748563cf83cf8a53a5ab569e7751bf158d9215f0e9b58b1b");

        let ethereum_address =
            recover_signer(&EcdsaSignature(signature), domain, &substrate_account).unwrap();
        assert_eq!(
            ethereum_address.0,
            hex!("6be02d1d3665660d22ff9624b7be0551ee1ac91b"),
        );
    }
}
