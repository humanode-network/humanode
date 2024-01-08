//! The validator key integration logic.

use std::{fmt::Display, sync::Arc};

use bioauth_flow_rpc::Signer;
use sp_application_crypto::AppPublic;
use sp_keystore::Keystore;

/// The validator public key implementation using the app crypto public key.
#[derive(Clone)]
pub struct AppCryptoPublic<T>(pub T);

/// The validator signer implementation using the keystore and app crypto public key.
pub struct AppCryptoSigner<PK> {
    /// The keystore to use for signing.
    pub keystore: Arc<dyn Keystore>,
    /// The public key to provide the signature for.
    pub public_key: AppCryptoPublic<PK>,
}

impl<PK> AppCryptoSigner<PK> {
    /// Create a new [`AppCryptoSigner`].
    pub fn new(keystore: Arc<dyn Keystore>, public_key: AppCryptoPublic<PK>) -> Self {
        Self {
            keystore,
            public_key,
        }
    }
}

/// An error that occurred at the signer.
#[derive(thiserror::Error, Debug)]
pub enum SignerError {
    /// The keystore error.
    #[error("keystore error: {0}")]
    Keystore(sp_keystore::Error),
    /// An error that occurred because the produced signature was `None`.
    #[error("unable to produce a signature")]
    NoSignature,
}

#[async_trait::async_trait]
impl<PK> Signer<Vec<u8>> for AppCryptoSigner<PK>
where
    PK: AppPublic,
{
    type Error = SignerError;

    async fn sign<'a, D>(&self, data: D) -> Result<Vec<u8>, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        let data = data.as_ref();
        let outcome = self
            .keystore
            .sign_with(PK::ID, PK::CRYPTO_ID, self.public_key.0.as_slice(), data)
            .map_err(SignerError::Keystore)?;

        outcome.ok_or(SignerError::NoSignature)
    }
}

impl<T> AsRef<[u8]> for AppCryptoPublic<T>
where
    T: AppPublic,
{
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<T> AppCryptoPublic<T>
where
    T: AppPublic,
{
    /// List all public keys in the keystore.
    pub async fn list(
        keystore: &dyn Keystore,
    ) -> Result<impl Iterator<Item = Self>, sp_keystore::Error> {
        let crypto_type_public_pairs = keystore.keys(T::ID)?;
        let filtered = crypto_type_public_pairs
            .into_iter()
            .filter_map(|public_key| match T::from_slice(&public_key) {
                Ok(id) => Some(Self(id)),
                Err(_) => None,
            });
        Ok(filtered)
    }
}

impl<T> Display for AppCryptoPublic<T>
where
    T: AppPublic,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use humanode_runtime::BioauthConsensusId;
    use sp_core::Decode;
    use sp_runtime::traits::TrailingZeroInput;

    use super::*;

    #[test]
    fn display() {
        let zero_id = BioauthConsensusId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
        let key = AppCryptoPublic(zero_id);
        assert_eq!(
            key.to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn display_does_not_match_raw_key() {
        let zero_id = BioauthConsensusId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
        assert_ne!(zero_id.to_string(), AppCryptoPublic(zero_id).to_string());
    }
}
