//! Keystore integration.

use std::marker::PhantomData;

use sp_application_crypto::{AppPublic, CryptoTypePublicPair};
use sp_keystore::SyncCryptoStorePtr;

/// Encapsulates validator public key extraction logic.
pub struct ValidatorKeyExtractor<Id, Selector> {
    /// Keystore to extract author.
    keystore: SyncCryptoStorePtr,
    /// The validator key selector.
    selector: Selector,
    /// The identity type.
    _phantom_id: PhantomData<Id>,
}

/// An error that can occur at validator public key extraction logic.
#[derive(Debug, thiserror::Error)]
pub enum ValidatorKeyExtractorError<SelectorError> {
    /// Something went wrong during the keystore interop.
    #[error("keystore error: {0}")]
    Keystore(sp_keystore::Error),
    /// The key was corrupted at the keystore.
    #[error("сorrupted public key - invalid size")]
    CorruptedKey,
    /// Something went wrong when selecting the key.
    #[error("key selection error: {0}")]
    Selector(SelectorError),
}

impl<Id, Selector> ValidatorKeyExtractor<Id, Selector> {
    /// Create a new [`ValidatorKeyExtractor`].
    pub fn new(keystore: SyncCryptoStorePtr, selector: Selector) -> Self {
        Self {
            keystore,
            selector,
            _phantom_id: PhantomData,
        }
    }
}

impl<Id, Selector> crate::ValidatorKeyExtractor for ValidatorKeyExtractor<Id, Selector>
where
    Id: AppPublic,
    Selector: crate::ValidatorKeySelector<Id>,
{
    type Error = ValidatorKeyExtractorError<Selector::Error>;
    type PublicKeyType = Id;

    fn extract_validator_key(&self) -> Result<Option<Self::PublicKeyType>, Self::Error> {
        let keystore_ref = self.keystore.as_ref();

        let crypto_type_public_pairs = tokio::task::block_in_place(move || {
            sp_keystore::SyncCryptoStore::keys(keystore_ref, Id::ID)
        })
        .map_err(ValidatorKeyExtractorError::Keystore)?;

        let matching_crypto_public_keys = crypto_type_public_pairs.into_iter().filter_map(
            |CryptoTypePublicPair(crypto_type_id, public_key)| {
                if crypto_type_id == Id::CRYPTO_ID {
                    Some(public_key)
                } else {
                    None
                }
            },
        );

        let matching_keys =
            matching_crypto_public_keys.filter_map(|bytes| Id::from_slice(&bytes).ok());

        let key = self
            .selector
            .select_key(matching_keys)
            .map_err(ValidatorKeyExtractorError::Selector)?;

        Ok(key)
    }
}

/// Selects one key out of one.
#[derive(Debug)]
pub struct OneOfOneSelector;

/// Multiple keys were found.
#[derive(Debug, thiserror::Error)]
#[error("We expect there to be no more than one key of a certain type and purpose")]
pub struct MultipleKeysError;

impl<Id> crate::ValidatorKeySelector<Id> for OneOfOneSelector {
    type Error = MultipleKeysError;

    fn select_key<T: Iterator<Item = Id>>(&self, mut keys: T) -> Result<Option<Id>, Self::Error> {
        let first_key = keys.next();

        // If there is more than one - return an error.
        if keys.next().is_some() {
            return Err(MultipleKeysError);
        }

        Ok(first_key)
    }
}
