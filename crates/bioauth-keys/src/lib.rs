//! The bioauth keys utils.

use std::marker::PhantomData;

use sp_application_crypto::{AppKey, CryptoTypePublicPair};
use sp_keystore::KeystorePtr;

pub mod traits;

/// Selects a particular key from the list of the keys available.
pub trait KeySelector<Key> {
    /// An error that renders key selection impossible.
    type Error;

    /// Select one key from the list of the keys.
    fn select_key<T: Iterator<Item = Key>>(&self, keys: T) -> Result<Option<Key>, Self::Error>;
}

/// Extracts a public key of a certain type from the keystore.
pub struct KeyExtractor<Id, Selector> {
    /// Keystore to extract author.
    keystore: KeystorePtr,
    /// The validator key selector.
    selector: Selector,
    /// The identity type.
    _phantom_id: PhantomData<Id>,
}

/// An error that can occur at public key extraction logic.
#[derive(Debug, thiserror::Error)]
pub enum KeyExtractorError<SelectorError> {
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

impl<Id, Selector> KeyExtractor<Id, Selector> {
    /// Create a new [`KeyExtractor`].
    pub fn new(keystore: KeystorePtr, selector: Selector) -> Self {
        Self {
            keystore,
            selector,
            _phantom_id: PhantomData,
        }
    }
}

impl<Id, Selector> traits::KeyExtractor for KeyExtractor<Id, Selector>
where
    Id: for<'a> TryFrom<&'a [u8]> + AppKey,
    Selector: KeySelector<Id>,
{
    type Error = KeyExtractorError<Selector::Error>;
    type PublicKeyType = Id;

    fn extract_key(&self) -> Result<Option<Self::PublicKeyType>, Self::Error> {
        let keystore_ref = self.keystore.as_ref();

        let crypto_type_public_pairs = sp_keystore::KeystorePtr::keys(keystore_ref, Id::ID)
            .map_err(KeyExtractorError::Keystore)?;

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
            matching_crypto_public_keys.filter_map(|bytes| Id::try_from(&bytes).ok());

        let key = self
            .selector
            .select_key(matching_keys)
            .map_err(KeyExtractorError::Selector)?;

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

impl<Id> KeySelector<Id> for OneOfOneSelector {
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
