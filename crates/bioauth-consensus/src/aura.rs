//! Aura consensus integration.

use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_consensus_aura::{digests::CompatibleDigestItem, AuraApi};
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::traits::{Block as BlockT, Header};
use std::{marker::PhantomData, sync::Arc};

/// Encapsulates block author extraction logic for aura consensus.
#[derive(Debug)]
pub struct BlockAuthorExtractor<Block: BlockT, Client, AuraAuthorityId> {
    /// Client provides access to the runtime.
    client: Arc<Client>,
    /// The type of the block used in the chain.
    _phantom_block: PhantomData<Block>,
    /// The type used an authority id in aura.
    _phantom_aura_authority_id: PhantomData<AuraAuthorityId>,
}

/// An error that can occur during block author extraction with the aura consensus.
#[derive(Debug, thiserror::Error)]
pub enum AuraBlockAuthorExtractorError {
    /// Unable to extract aura authorities from the chain state via the runtime.
    #[error("unable to extract aura authorities: {0}")]
    UnableToExtractAuthorities(sp_api::ApiError),
    /// Unable to obtain the slot from the block header.
    #[error("unable to obtaion the slot from the block header")]
    UnableToObtainSlot,
}

impl<Block: BlockT, Client, AuraAuthorityId> BlockAuthorExtractor<Block, Client, AuraAuthorityId> {
    /// Create a new [`AuraBlockAuthorExtractor`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_block: PhantomData,
            _phantom_aura_authority_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, AuraAuthorityId> Clone
    for BlockAuthorExtractor<Block, Client, AuraAuthorityId>
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_block: PhantomData,
            _phantom_aura_authority_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, AuraAuthorityId> crate::BlockAuthorExtractor
    for BlockAuthorExtractor<Block, Client, AuraAuthorityId>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    Client::Api: AuraApi<Block, AuraAuthorityId>,
    AuraAuthorityId: codec::Codec + Clone,
{
    type Error = AuraBlockAuthorExtractorError;
    type Block = Block;
    type PublicKeyType = AuraAuthorityId;

    fn extract_block_author(
        &self,
        at: &BlockId<Self::Block>,
        block_header: &<Self::Block as BlockT>::Header,
    ) -> Result<AuraAuthorityId, Self::Error> {
        // Extract aura authorities list.
        let authorities = self
            .client
            .runtime_api()
            .authorities(at)
            .map_err(AuraBlockAuthorExtractorError::UnableToExtractAuthorities)?;

        // Extract the slot of a block.
        let slot = block_header
            .digest()
            .logs()
            .iter()
            .find_map(CompatibleDigestItem::<()>::as_aura_pre_digest)
            .ok_or(AuraBlockAuthorExtractorError::UnableToObtainSlot)?;

        // Author index in aura is current slot number mod authories.
        let author_index = *slot % authorities.len() as u64;

        // Determine the author of a block.
        let author_public_key = authorities
            .get(author_index as usize)
            .expect("author index is mod authories list len; qed");

        Ok(author_public_key.clone())
    }
}

/// Encapsulates validator public key extraction logic for aura consensus.
pub struct ValidatorKeyExtractor {
    /// Keystore to extract validator public key.
    keystore: SyncCryptoStorePtr,
}

impl ValidatorKeyExtractor {
    /// Create a new [`ValidatorKeyExtractor`].
    pub fn new(keystore: SyncCryptoStorePtr) -> Self {
        Self { keystore }
    }
}

impl crate::ValidatorKeyExtractor for ValidatorKeyExtractor {
    type PublicKeyType = sp_consensus_aura::sr25519::AuthorityId;

    fn extract_validator_key(&self) -> Self::PublicKeyType {
        let keystore_ref = self.keystore.as_ref();
        let validator_public_keys = tokio::task::block_in_place(move || {
            sp_keystore::SyncCryptoStore::sr25519_public_keys(
                keystore_ref,
                sp_application_crypto::key_types::AURA,
            )
        });

        assert!(
            validator_public_keys.len() == 1,
            "The list of validator public keys should contain only 1 key, please report this"
        );

        validator_public_keys[0].into()
    }
}
