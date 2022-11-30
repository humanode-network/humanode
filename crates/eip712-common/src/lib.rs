//! Common logic for EIP-712 typed data message construction and signature verification.

#![cfg_attr(not(feature = "std"), no_std)]

pub use primitives_ethereum::{EcdsaSignature, EthereumAddress};
pub use sp_core_hashing_proc_macro::keccak_256 as const_keccak_256;
pub use sp_io::hashing::keccak_256;

/// `EIP712Domain` typehash.
const EIP712_DOMAIN_TYPEHASH: [u8; 32] = const_keccak_256!(
    b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
);

/// A type alias representing a `string` solidity type.
pub type EthString = str;
/// A type alias representing a `uint256` solidity type.
pub type EthUint265 = [u8; 32];
/// A type alias representing an `address` solidity type.
pub type EthAddress = [u8; 20];
/// A type alias representing the `bytes` solidity type.
pub type EthBytes = [u8];

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

/// Prepare a hash for `EIP712Domain` data type.
fn make_domain_hash(domain: Domain<'_>) -> [u8; 32] {
    let mut buf = [0u8; 160];
    buf[0..32].copy_from_slice(&EIP712_DOMAIN_TYPEHASH);
    buf[32..64].copy_from_slice(&keccak_256(domain.name.as_bytes()));
    buf[64..96].copy_from_slice(&keccak_256(domain.version.as_bytes()));
    buf[96..128].copy_from_slice(domain.chain_id);
    buf[140..160].copy_from_slice(domain.verifying_contract);
    keccak_256(&buf)
}

/// Prepare a hash for the payload.
/// To be used at EIP-712 message payload.
pub fn make_payload_hash<'a>(
    typehash: &[u8; 32],
    datahashes: impl IntoIterator<Item = &'a [u8; 32]>,
) -> [u8; 32] {
    let datahashes = datahashes.into_iter();
    let (datahashes_size, _) = datahashes.size_hint();
    let mut buf = sp_std::prelude::Vec::with_capacity(32 + datahashes_size * 32);
    buf.extend_from_slice(typehash);
    for item in datahashes {
        buf.extend_from_slice(item);
    }
    keccak_256(&buf)
}

/// Prepare the EIP-712 message.
pub fn make_message_hash(domain: Domain<'_>, payload_hash: &[u8; 32]) -> [u8; 32] {
    let domain_hash = make_domain_hash(domain);
    make_eip712_message_hash(&domain_hash, payload_hash)
}

/// Extract the signer address from the signature and the message.
pub fn recover_signer(sig: &EcdsaSignature, msg: &[u8; 32]) -> Option<EthereumAddress> {
    let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig.0, msg).ok()?;
    Some(ecdsa_public_key_to_ethereum_address(&pubkey))
}

/// Verify EIP-712 typed signature based on provided domain and payload hash.
pub fn verify_signature(
    signature: &EcdsaSignature,
    domain: Domain<'_>,
    payload_hash: &[u8; 32],
) -> Option<EthereumAddress> {
    let msg_hash = make_message_hash(domain, payload_hash);
    recover_signer(signature, &msg_hash)
}

/// Convert the ECDSA public key to Ethereum address.
fn ecdsa_public_key_to_ethereum_address(pubkey: &[u8; 64]) -> EthereumAddress {
    let mut address = [0u8; 20];
    address.copy_from_slice(&sp_io::hashing::keccak_256(pubkey)[12..]);
    EthereumAddress(address)
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use sp_core::U256;

    use super::*;

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

    #[test]
    fn verify_test_chain_id() {
        let domain = prepare_sample_domain();
        let expected_chain_id: [u8; 32] = U256::from(5234).into();
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
}
