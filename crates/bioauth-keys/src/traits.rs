//! Generic traits to abstract away the implementations.

/// [`KeyExtractor`] provides functionality to extract the key of the node from the keystore.
pub trait KeyExtractor {
    /// An error that can occur during the key extraction.
    type Error: std::error::Error;
    /// The public key type that the uses.
    type PublicKeyType;

    /// Extract public key.
    fn extract_key(&self) -> Result<Option<Self::PublicKeyType>, Self::Error>;
}

impl<T: KeyExtractor> KeyExtractor for std::sync::Arc<T> {
    type Error = T::Error;
    type PublicKeyType = T::PublicKeyType;

    fn extract_key(&self) -> Result<Option<Self::PublicKeyType>, Self::Error> {
        self.as_ref().extract_key()
    }
}
