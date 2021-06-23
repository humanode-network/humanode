#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod ed25519 {

    /// An Robonode keypair using Ed25519 as its crypto.
    pub type RobonodePair = sp_application_crypto::ed25519::Pair;

    /// An Robonode signature using Ed25519 as its crypto.
    pub type RobonodeSignature = sp_application_crypto::ed25519::Signature;

    /// An Robonode identifier using Ed25519 as its crypto.
    pub type RobonodePublicKey = sp_application_crypto::ed25519::Public;
}

#[cfg(test)]
mod tests {
    use crate::ed25519::{RobonodePair, RobonodePublicKey, RobonodeSignature};
    use hex_literal::hex;
    use sp_application_crypto::Pair;

    #[test]
    fn generated_pair_should_work() {
        let pair = RobonodePair::generate();
        let public = pair.0.public();
        let message = b"Something important";
        let signature = pair.0.sign(&message[..]);
        assert!(RobonodePair::verify(&signature, &message[..], &public));
        assert!(!RobonodePair::verify(
            &signature,
            b"Something else",
            &public
        ));
    }

    #[test]
    fn test_vector_by_string_should_work() {
        let pair = RobonodePair::from_string(
            "0x9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
            None,
        )
        .unwrap();
        let public = pair.public();
        assert_eq!(
            public,
            RobonodePublicKey::from_raw(hex!(
                "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
            ))
        );
        let message = b"";
        let signature = hex!("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b");
        let signature = RobonodeSignature::from_raw(signature);
        assert!(pair.sign(&message[..]) == signature);
        assert!(RobonodePair::verify(&signature, &message[..], &public));
    }
}
