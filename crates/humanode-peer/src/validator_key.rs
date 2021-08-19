//! The validator key integration logic.

use std::{fmt::Display, sync::Arc};

use bioauth_flow::flow::Signer;
use sp_application_crypto::Public;
use sp_keystore::CryptoStore;

/// The validator public key implementation using the aura public key.
pub struct AuraPublic(pub sp_application_crypto::sr25519::Public);

/// The validator signer implementation using the keystore and aura public key.
pub struct AuraSigner {
    /// The keystore to use for signing.
    pub keystore: Arc<dyn CryptoStore>,
    /// The public key to provide the signature for.
    pub public_key: AuraPublic,
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
impl Signer<Vec<u8>> for AuraSigner {
    type Error = SignerError;

    async fn sign<'a, D>(&self, data: D) -> Result<Vec<u8>, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        let data = data.as_ref();
        let outcome = self
            .keystore
            .sign_with(
                sp_application_crypto::key_types::AURA,
                &self.public_key.0.to_public_crypto_pair(),
                data,
            )
            .await
            .map_err(SignerError::Keystore)?;

        outcome.ok_or(SignerError::NoSignature)
    }
}

impl AsRef<[u8]> for AuraPublic {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AuraPublic {
    /// Fetch the aura public key from the keystore.
    pub async fn from_keystore(keystore: &dyn CryptoStore) -> Option<Self> {
        let mut list = Self::list(keystore).await;
        assert!(
            list.size_hint().0 <= 1,
            "The list of aura public keys is larger than 1; please report this"
        );
        list.next()
    }

    /// List all [`AuraPublic`] keys in the keystore.
    pub async fn list(keystore: &dyn CryptoStore) -> impl Iterator<Item = Self> {
        let aura_public_keys = keystore
            .sr25519_public_keys(sp_application_crypto::key_types::AURA)
            .await;
        aura_public_keys.into_iter().map(Self)
    }
}

impl Display for AuraPublic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
