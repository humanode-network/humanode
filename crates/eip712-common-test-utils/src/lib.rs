//! Common test utils for EIP-712 typed data message construction and signature verification.

use eip712_common::*;
use ethers::types::transaction::eip712::{Eip712, TypedData};
use primitives_ethereum::{EcdsaSignature, EthereumAddress};
pub use sp_core::{crypto::Pair, ecdsa, H256, U256};

/// Create a new ECDSA keypair from the seed.
pub fn ecdsa_pair(seed: &[u8]) -> ecdsa::Pair {
    ecdsa::Pair::from_seed(&keccak_256(seed))
}

/// Sign a given message with the given ECDSA keypair.
pub fn ecdsa_sign(pair: &ecdsa::Pair, msg: &[u8; 32]) -> EcdsaSignature {
    EcdsaSignature(pair.sign_prehashed(msg).0)
}

/// Sign a given EIP-712 typed data JSON with the given ECDSA keypair.
pub fn ecdsa_sign_typed_data(pair: &ecdsa::Pair, type_data_json: &str) -> EcdsaSignature {
    let typed_data: TypedData = serde_json::from_str(type_data_json).unwrap();
    let msg_bytes: [u8; 32] = typed_data.encode_eip712().unwrap();
    ecdsa_sign(pair, &msg_bytes)
}

/// Create an Ethereum address from the given ECDSA keypair.
pub fn ethereum_address(pair: &ecdsa::Pair) -> EthereumAddress {
    let public = secp256k1::PublicKey::from_slice(&pair.public().0).unwrap();
    let mut public_bytes = [0u8; 64];
    public_bytes.copy_from_slice(&public.serialize_uncompressed()[1..]);
    let mut address = [0u8; 20];
    address.copy_from_slice(&keccak_256(&public_bytes)[12..]);
    EthereumAddress(address)
}

/// Create an Ethereum address from the given seed.
///
/// This algorithm will return the addresses corresponding to the [`ecdsa::Pair`]s generated
/// by [`ecdsa_pair`] with the same `seed`.
pub fn ethereum_address_from_seed(seed: &[u8]) -> EthereumAddress {
    let pair = ecdsa_pair(seed);
    ethereum_address(&pair)
}
