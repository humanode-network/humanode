//! Keystore integration.

use std::marker::PhantomData;

use sp_application_crypto::{AppPublic, CryptoTypePublicPair};
use sp_keystore::SyncCryptoStorePtr;

/// Encapsulates validator public key extraction logic.
pub struct ValidatorKeyExtractor<Id> {
    /// Keystore to extract author.
    keystore: SyncCryptoStorePtr,
    /// The identity type.
    _phantom_id: PhantomData<Id>,
}

/// An error that can occur at validator public key extraction logic.
#[derive(Debug, thiserror::Error)]
pub enum ValidatorKeyExtractorError {
    /// Something went wrong during the keystore interop.
    #[error("keystore error: {0}")]
    Keystore(sp_keystore::Error),
    /// The key was corrupted at the keystore.
    #[error("сorrupted public key - invalid size")]
    CorruptedKey,
}

impl<Id> ValidatorKeyExtractor<Id> {
    /// Create a new [`ValidatorKeyExtractor`].
    pub fn new(keystore: SyncCryptoStorePtr) -> Self {
        Self {
            keystore,
            _phantom_id: PhantomData,
        }
    }
}

impl<Id> crate::ValidatorKeyExtractor for ValidatorKeyExtractor<Id>
where
    Id: AppPublic,
{
    type Error = ValidatorKeyExtractorError;
    type PublicKeyType = Id;

    fn extract_validator_key(&self) -> Result<Option<Self::PublicKeyType>, Self::Error> {
        let keystore_ref = self.keystore.as_ref();

        let crypto_type_public_pairs = tokio::task::block_in_place(move || {
            sp_keystore::SyncCryptoStore::keys(keystore_ref, Id::ID)
        })
        .map_err(ValidatorKeyExtractorError::Keystore)?;

        let mut matching_crypto_public_keys = crypto_type_public_pairs.into_iter().filter_map(
            |CryptoTypePublicPair(crypto_type_id, public_key)| {
                if crypto_type_id == Id::CRYPTO_ID {
                    Some(public_key)
                } else {
                    None
                }
            },
        );

        let first_key = matching_crypto_public_keys.next();

        // If there is more than one - we'll need to handle it somehow.
        assert!(
            matching_crypto_public_keys.next().is_none(),
            "We expect there to be no more than one key of a certain type and purpose, please report this"
        );

        let key = match first_key {
            Some(bytes) => {
                Some(Id::from_slice(&bytes).map_err(|_| ValidatorKeyExtractorError::CorruptedKey)?)
            }
            None => None,
        };

        Ok(key)
    }
}
