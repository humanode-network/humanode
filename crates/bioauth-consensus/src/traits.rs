//! Traits for abstracting away the integration with the underlying consensus.

use sp_api::BlockId;
use sp_runtime::traits::Block as BlockT;

/// BlockAuthorExtractor provides functionality to extract
/// block author public key for a particular block.
pub trait BlockAuthorExtractor {
    /// An error that can occur during block author extraction.
    type Error;
    /// The block type used in the chain.
    type Block: BlockT;
    /// The public key type used by the block authors to sign blocks.
    type PublicKeyType;

    /// Extract the public key used to sign the block from the specified block header.
    fn extract_block_author(
        &self,
        at: &BlockId<Self::Block>,
        block_header: &<Self::Block as BlockT>::Header,
    ) -> Result<Self::PublicKeyType, Self::Error>;
}

/// AuthorizationVerifier provides a functionality to verify
/// whether a particular author is authorized to be a validator.
pub trait AuthorizationVerifier {
    /// An error that can occur during the authrization verification.
    type Error;
    /// The block type used in the chain.
    type Block: BlockT;
    /// The public key that is used for block signing by validators.
    type PublicKeyType: ?Sized;

    /// Verify that a provided validator public key is authorized for the purposes to produce
    /// blocks for the bioauth purposes.
    fn is_authorized(
        &self,
        at: &BlockId<Self::Block>,
        public_key: &Self::PublicKeyType,
    ) -> Result<bool, Self::Error>;
}

/// ValidatorKeyExtractor provides functionality to extract the validator key of the peer.
pub trait ValidatorKeyExtractor {
    /// An error that can occur during the validator key extraction.
    type Error;
    /// The public key type that the validator uses.
    type PublicKeyType;

    /// Extract validator public key.
    fn extract_validator_key(&self) -> Result<Option<Self::PublicKeyType>, Self::Error>;
}

impl<T: ValidatorKeyExtractor> ValidatorKeyExtractor for std::sync::Arc<T> {
    type Error = T::Error;
    type PublicKeyType = T::PublicKeyType;

    fn extract_validator_key(&self) -> Result<Option<Self::PublicKeyType>, Self::Error> {
        self.as_ref().extract_validator_key()
    }
}
