//! The validator key integration logic.

use std::{fmt::Display, sync::Arc};

use bioauth_flow::Signer;
use sp_application_crypto::{AppPublic, CryptoTypePublicPair};
use sp_keystore::CryptoStore;

/// The validator public key implementation using the app crypto public key.
#[derive(Clone)]
pub struct AppCryptoPublic<T>(pub T);

/// The validator signer implementation using the keystore and app crypto public key.
pub struct AppCryptoSigner<PK> {
    /// The keystore to use for signing.
    pub keystore: Arc<dyn CryptoStore>,
    /// The public key to provide the signature for.
    pub public_key: AppCryptoPublic<PK>,
}

impl<PK> AppCryptoSigner<PK> {
    /// Create a new [`AppCryptoSigner`].
    pub fn new(keystore: Arc<dyn CryptoStore>, public_key: AppCryptoPublic<PK>) -> Self {
        Self {
            keystore,
            public_key,
        }
    }
}

/// An error that occured at the signer.
#[derive(thiserror::Error, Debug)]
pub enum SignerError {
    /// The keystore error.
    #[error("keystore error: {0}")]
    Keystore(sp_keystore::Error),
    /// An error that occured because the produced signature was `None`.
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
            .sign_with(PK::ID, &self.public_key.0.to_public_crypto_pair(), data)
            .await
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
    /// Fetch the public key from the keystore.
    pub async fn from_keystore(
        keystore: &dyn CryptoStore,
    ) -> Result<Option<Self>, sp_keystore::Error> {
        let mut list = Self::list(keystore).await?;
        let first = list.next();
        assert!(
            list.next().is_none(),
            "The list of public keys is larger than 1; please report this"
        );
        Ok(first)
    }

    /// List all public keys in the keystore.
    pub async fn list(
        keystore: &dyn CryptoStore,
    ) -> Result<impl Iterator<Item = Self>, sp_keystore::Error> {
        let crypto_type_public_pairs = keystore.keys(T::ID).await?;
        let filtered = crypto_type_public_pairs.into_iter().filter_map(
            |CryptoTypePublicPair(crypto_type, public_key)| {
                if crypto_type == T::CRYPTO_ID {
                    Some(Self(T::from_slice(&public_key)))
                } else {
                    None
                }
            },
        );
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
    use super::*;
    use humanode_runtime::BioauthConsensusId;

    #[test]
    fn display() {
        let key = AppCryptoPublic(BioauthConsensusId::default());
        assert_eq!(
            key.to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn display_does_not_match_raw_key() {
        let key = BioauthConsensusId::default();
        assert_ne!(key.to_string(), AppCryptoPublic(key).to_string());
    }
}
