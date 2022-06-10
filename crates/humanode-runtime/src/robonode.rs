use super::*;

/// The wrapper for the robonode public key, that enables storing it in the state.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    pub fn from_bytes(
        bytes: &[u8],
    ) -> Result<Self, robonode_crypto::ed25519_dalek::ed25519::Error> {
        let actual_key = robonode_crypto::PublicKey::from_bytes(bytes)?;
        Ok(Self(actual_key.to_bytes()))
    }
}

/// The error that can occur during robonode signature validation.
pub enum PublicKeyError {
    UnableToParseKey,
    UnableToParseSignature,
    UnableToValidateSignature(robonode_crypto::ed25519_dalek::ed25519::Error),
}

impl pallet_bioauth::Verifier<Vec<u8>> for PublicKey {
    type Error = PublicKeyError;

    fn verify<'a, D>(&self, data: D, signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        use robonode_crypto::Verifier;

        let actual_key = robonode_crypto::PublicKey::from_bytes(&self.0)
            .map_err(|_| PublicKeyError::UnableToParseKey)?;

        let signature: robonode_crypto::Signature = signature
            .as_slice()
            .try_into()
            .map_err(|_| PublicKeyError::UnableToParseSignature)?;

        actual_key
            .verify(data.as_ref(), &signature)
            .map_err(PublicKeyError::UnableToValidateSignature)?;

        Ok(true)
    }
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks {
    use super::*;

    fn derive_keypair_from_secret_key(secret_key_bytes: [u8; 32]) -> robonode_crypto::Keypair {
        // Derive Public Key component.
        let robonode_secret_key =
            robonode_crypto::SecretKey::from_bytes(secret_key_bytes.as_ref()).unwrap();
        let robonode_public_key: robonode_crypto::PublicKey = (&robonode_secret_key).into();

        // Constructs bytes of Secret Key and Public Key.
        let mut keypair_bytes = [0; 64];
        let _ = &keypair_bytes[..32].copy_from_slice(robonode_secret_key.as_bytes());
        let _ = &keypair_bytes[32..].copy_from_slice(robonode_public_key.as_bytes());

        robonode_crypto::Keypair::from_bytes(keypair_bytes.as_ref()).unwrap()
    }

    impl pallet_bioauth::benchmarking::AuthTicketSigner for Runtime {
        fn sign(auth_ticket: &[u8]) -> Vec<u8> {
            use robonode_crypto::{Signature, Signer};
            // This secret key is taken from the first entry in https://ed25519.cr.yp.to/python/sign.input.
            // Must be compatible with public key provided in benchmark_config() function in
            // crates/humanode-peer/src/chain_spec.rs
            const ROBONODE_SECRET_KEY: [u8; 32] = hex_literal::hex!(
                "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
            );
            let robonode_keypair = derive_keypair_from_secret_key(ROBONODE_SECRET_KEY);
            robonode_keypair
                .try_sign(auth_ticket)
                .unwrap_or(Signature::from_bytes(&[0; 64]).unwrap())
                .to_bytes()
                .to_vec()
        }
    }

    impl pallet_bioauth::benchmarking::RobonodePublicKeyBuilder<robonode::PublicKey> for Runtime {
        fn build(value: pallet_bioauth::benchmarking::RobonodePublicKeyBuilderValue) -> PublicKey {
            match value {
                pallet_bioauth::benchmarking::RobonodePublicKeyBuilderValue::A => {
                    robonode::PublicKey(hex_literal::hex!(
                        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
                    ))
                }
                pallet_bioauth::benchmarking::RobonodePublicKeyBuilderValue::B => {
                    robonode::PublicKey(hex_literal::hex!(
                        "0000000000000000000000000000000000000000000000000000000000000000"
                    ))
                }
            }
        }
    }
}
