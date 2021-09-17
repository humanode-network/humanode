//! Babe consensus integration.

use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_consensus_babe::BabeApi;
use sp_runtime::traits::Block as BlockT;
use std::{marker::PhantomData, sync::Arc};

/// Encapsulates block author extraction logic for the babe consensus.
#[derive(Debug)]
pub struct BlockAuthorExtractor<Block: BlockT, Client> {
    /// Client provides access to the runtime.
    client: Arc<Client>,
    /// The type of the block used in the chain.
    _phantom_block: PhantomData<Block>,
}

/// An error that can occur during block author extraction with the babe consensus.
#[derive(Debug, thiserror::Error)]
pub enum BlockAuthorExtractorError {
    /// Unable to extract babe authorities for current epoch from the chain state via the runtime.
    #[error("unable to extract babe authorities for current epoch: {0}")]
    UnableToExtractAuthorities(sp_api::ApiError),
    /// Unable to obtain the author from the block header.
    #[error("unable to obtaion the author from the block header")]
    UnableToObtainAuthor,
}

impl<Block: BlockT, Client> BlockAuthorExtractor<Block, Client> {
    /// Create a new [`BlockAuthorExtractor`].
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
    Client::Api: BabeApi<Block>,
{
    type Error = BlockAuthorExtractorError;
    type Block = Block;
    type PublicKeyType = sp_consensus_babe::AuthorityId;

    fn extract_block_author(
        &self,
        at: &BlockId<Self::Block>,
        block_header: &<Self::Block as BlockT>::Header,
    ) -> Result<Self::PublicKeyType, Self::Error> {
        let current_epoch = self
            .client
            .runtime_api()
            .current_epoch(at)
            .map_err(BlockAuthorExtractorError::UnableToExtractAuthorities)?;
        let authorities = &current_epoch.authorities;

        let pre_digest = sc_consensus_babe::find_pre_digest::<Block>(block_header).unwrap();

        let author = match authorities.get(pre_digest.authority_index() as usize) {
            Some(author) => author.0.clone(),
            None => return Err(BlockAuthorExtractorError::UnableToObtainAuthor),
        };

        Ok(author)
    }
}
