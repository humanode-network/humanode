//! Block author validation logic.

/// BlockAuthorExtractor provides a functionality to extract
/// block author public key for a particular block.
pub trait BlockAuthorExtractor {
    /// BlockAuthorExtractor error.
    type Error;
    /// BlockHeader type.
    type BlockHeader;
    /// Block author Public key type.
    type PublicKeyType;

    /// Extract block author public key for a provided block header.
    fn extract_block_author(
        &self,
        block_header: &Self::BlockHeader,
    ) -> Result<Self::PublicKeyType, Self::Error>;
}

/// BlockAuthorExtractor provides a functionality to verify
/// whether aparticular author is authorized to be a validator.
pub trait AuthorizationVerifier {
    /// AuthorizationVerifier
    type Error;
    /// Public key type.
    type PublicKeyType: ?Sized;

    /// Verify that a provided author is authorized by it's public key.
    fn is_authorized(&self, author_public_key: &Self::PublicKeyType) -> Result<bool, Self::Error>;
}
