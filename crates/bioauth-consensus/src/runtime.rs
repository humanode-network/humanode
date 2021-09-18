//! Runtime integration.

use sp_api::{BlockId, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;
use std::{marker::PhantomData, sync::Arc};

/// Encapsulates block author extraction logic.
#[derive(Debug)]
pub struct BlockAuthorExtractor<Block: BlockT, Client, AuthorityId> {
    /// Client provides access to the runtime.
    client: Arc<Client>,
    /// The type of the block used in the chain.
    _phantom_block: PhantomData<Block>,
    /// The type used an authority id.
    _phantom_authority_id: PhantomData<AuthorityId>,
}

/// An error that can occur during block author extraction.
#[derive(Debug, thiserror::Error)]
pub enum BlockAuthorExtractorError {
    /// An error occured duraing an attempt to extract the block author from the chain state via
    /// runtime.
    #[error("unable to extract block author: {0}")]
    UnableToExtractBlockAuthor(sp_api::ApiError),
    /// The author extraction call returned no author, so we are unable to perform bioauth.
    #[error("block author not found, the block is not bioauth-authorized")]
    BlockAuthorNotFound,
}

impl<Block: BlockT, Client, AuthorityId> BlockAuthorExtractor<Block, Client, AuthorityId> {
    /// Create a new [`BlockAuthorExtractor`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_block: PhantomData,
            _phantom_authority_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, AuthorityId> Clone
    for BlockAuthorExtractor<Block, Client, AuthorityId>
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_block: PhantomData,
            _phantom_authority_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, AuthorityId> crate::BlockAuthorExtractor
    for BlockAuthorExtractor<Block, Client, AuthorityId>
where
    Client: ProvideRuntimeApi<Block>,
    Client::Api: BlockAuthorship<Block, AuthorityId>,
    AuthorityId: codec::Codec + Clone,
{
    type Error = BlockAuthorExtractorError;
    type Block = Block;
    type PublicKeyType = AuthorityId;

    fn extract_block_author(
        &self,
        at: &BlockId<Self::Block>,
        _block_header: &<Self::Block as BlockT>::Header,
    ) -> Result<AuthorityId, Self::Error> {
        let author = self
            .client
            .runtime_api()
            .author(at)
            .map_err(BlockAuthorExtractorError::UnableToExtractBlockAuthor)?;

        let author = author.ok_or(BlockAuthorExtractorError::BlockAuthorNotFound)?;

        Ok(author)
    }
}
