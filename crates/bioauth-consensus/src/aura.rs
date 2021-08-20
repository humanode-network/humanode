//! Aura consensus integration.

use sp_api::{BlockId, Decode, ProvideRuntimeApi};
use sp_application_crypto::Public;
use sp_blockchain::HeaderBackend;
use sp_consensus_aura::{AuraApi, Slot};
use sp_runtime::generic::OpaqueDigestItemId;
use sp_runtime::traits::{Block as BlockT, Header};
use std::{marker::PhantomData, sync::Arc};

/// Encapsulates block author extraction logic for aura consensus.
#[derive(Debug)]
pub struct BlockAuthorExtractor<Block: BlockT, Client> {
    /// Client provides access to the runtime.
    client: Arc<Client>,
    /// The type from the block used in the chain.
    _phantom_block: PhantomData<Block>,
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
    /// Unable to decode the slot.
    #[error("unable to decode the slot")]
    UnableToDecodeSlot,
}

impl<Block: BlockT, Client> BlockAuthorExtractor<Block, Client> {
    /// Create a new [`AuraBlockAuthorExtractor`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_block: PhantomData,
        }
    }
}

impl<Block: BlockT, Client> Clone for BlockAuthorExtractor<Block, Client> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_block: PhantomData,
        }
    }
}

impl<Block: BlockT, Client> crate::BlockAuthorExtractor for BlockAuthorExtractor<Block, Client>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    Client::Api: AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
{
    type Error = AuraBlockAuthorExtractorError;
    type Block = Block;
    type PublicKeyType = Vec<u8>;

    fn extract_block_author(
        &self,
        at: &BlockId<Self::Block>,
        block_header: &<Self::Block as BlockT>::Header,
    ) -> Result<Vec<u8>, Self::Error> {
        // Extract aura authorities list.
        let authorities = self
            .client
            .runtime_api()
            .authorities(at)
            .map_err(AuraBlockAuthorExtractorError::UnableToExtractAuthorities)?;

        // Extract the slot of a block.
        let mut slot = block_header
            .digest()
            .log(|l| l.try_as_raw(OpaqueDigestItemId::PreRuntime(b"aura")))
            .ok_or(AuraBlockAuthorExtractorError::UnableToObtainSlot)?;

        // Decode slot number.
        let slot_decoded = Slot::decode(&mut slot)
            .map_err(|_| AuraBlockAuthorExtractorError::UnableToDecodeSlot)?;

        // Author index in aura is current slot number mod authories.
        let author_index = *slot_decoded % authorities.len() as u64;

        // Determine the author of a block.
        let author_public_key = authorities
            .get(author_index as usize)
            .expect("author index is mod authories list len; qed");

        Ok(author_public_key.to_raw_vec())
    }
}
