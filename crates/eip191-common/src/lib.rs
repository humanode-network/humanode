//! Common logic for eth personal sign message construction and signature verification.

#![cfg_attr(not(feature = "std"), no_std)]

use numtoa::NumToA;
pub use primitives_ethereum::{EcdsaSignature, EthereumAddress};
pub use sp_core_hashing_proc_macro::keccak_256 as const_keccak_256;
pub use sp_io::hashing::keccak_256;
use sp_std::vec;

/// Prepare the eth personal sign message.
pub fn make_personal_message_hash(message: &[u8]) -> [u8; 32] {
    let mut buf = vec![];
    buf.extend_from_slice("\x19Ethereum Signed Message:\n".as_bytes());
    buf.extend_from_slice(usize_as_string_bytes(message.len()).as_slice());
    buf.extend_from_slice(message);
    keccak_256(&buf)
}

/// A helper function to represent message len as string bytes.
///
/// https://crates.io/crates/numtoa.
fn usize_as_string_bytes(message_len: usize) -> Vec<u8> {
    let mut buffer = [0u8; 20];
    message_len.numtoa(10, &mut buffer).to_vec()
}

/// Extract the signer address from the signature and the message.
pub fn recover_signer(sig: &EcdsaSignature, msg: &[u8; 32]) -> Option<EthereumAddress> {
    let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig.0, msg).ok()?;
    Some(ecdsa_public_key_to_ethereum_address(&pubkey))
}

/// Verify signature based on provided message.
pub fn verify_signature(signature: &EcdsaSignature, message: &[u8]) -> Option<EthereumAddress> {
    let msg_hash = make_personal_message_hash(message);
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
    use super::*;

    /// This test contains the data obtained from Metamask/eth-sig-util.
    ///
    /// https://github.com/MetaMask/eth-sig-util/blob/8a470650074174f5338308d2acbd97caf5542434/src/personal-sign.test.ts#L88
    #[test]
    fn valid_signature() {
        let message = "hello world";
        let hex_signature = "0xce909e8ea6851bc36c007a0072d0524b07a3ff8d4e623aca4c71ca8e57250c4d0a3fc38fa8fbaaa81ead4b9f6bd03356b6f8bf18bccad167d78891636e1d69561b";
        let expected_address = "0xbe93f9bacbcffc8ee6663f2647917ed7a20a57bb";

        let address = verify_signature(
            &EcdsaSignature(
                hex::decode(&hex_signature[2..])
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ),
            message.as_bytes(),
        );

        assert_eq!(
            format!("0x{}", hex::encode(address.unwrap().0)),
            expected_address
        );
    }

    /// This test contains the data obtained from MetaMask browser extension via an injected web3
    /// interface using personal_sign API.
    ///
    /// https://metamask.github.io/test-dapp/.
    ///
    /// It validates that the real-world external ecosystem works properly with our code.
    #[test]
    fn real_world_case() {
        let message = "Example `personal_sign` message";
        let hex_signature = "0xbef8374833e572271b2f17d233a8e03c53c8f35e451cd33494793bbdc036f1d72dd955c0628483bc50bd3f7849d1d730a69cdd9775ab3eed556b87eaa20426511b";
        let expected_address = "0xc16fb04cbc2c946399772688c33d9bb6ae6ac71b";

        let address = verify_signature(
            &EcdsaSignature(
                hex::decode(&hex_signature[2..])
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ),
            message.as_bytes(),
        );

        assert_eq!(
            format!("0x{}", hex::encode(address.unwrap().0)),
            expected_address
        );
    }
}
