//! Cryptographic primitives used by the robonode.

#![cfg_attr(not(feature = "std"), no_std)]

pub use ed25519_dalek::{self, Signer, Verifier};

/// Robonode signing key.
pub type SigningKey = ed25519_dalek::SigningKey;

/// Robonode signature.
pub type Signature = ed25519_dalek::Signature;

/// Robonode public key.
pub type PublicKey = ed25519_dalek::VerifyingKey;

/// Robonode secret key.
pub type SecretKey = ed25519_dalek::SecretKey;

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use hex_literal::hex;
    use rand::rngs::OsRng;

    use super::*;

    // Test vectors.
    // See: https://ed25519.cr.yp.to/python/sign.py
    //      https://ed25519.cr.yp.to/python/sign.input
    const SK: [u8; 32] = hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
    const PK: [u8; 32] = hex!("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a");
    const M: [u8; 0] = hex!("");
    const SM: [u8; 64] = hex!("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b");

    #[test]
    fn test_vectors() {
        let secret = SigningKey::from_bytes(&SK);
        let public = PublicKey::from_bytes(&PK).unwrap();
        assert_eq!(secret.verifying_key(), public);

        let signature: Signature = SM.into();

        println!("{:#02X?}", secret.sign(&M[..]).to_bytes());
        assert!(secret.sign(&M[..]) == signature);
        assert!(public.verify(&M[..], &signature).is_ok());
    }

    #[test]
    fn generated_pair() {
        let mut csprng = OsRng {};
        let pair = SigningKey::generate(&mut csprng);

        let message = b"Something important";
        let signature = pair.sign(&message[..]);

        assert!(pair.verify(&message[..], &signature).is_ok());
        assert!(pair.verify(b"Something else", &signature).is_err());
    }
}
