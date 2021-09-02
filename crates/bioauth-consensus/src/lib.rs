//! A block-import handler for Bioauth.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use sc_client_api::{backend::Backend, Finalizer};
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{ProvideRuntimeApi, TransactionFor};
use sp_blockchain::{well_known_cache_keys, HeaderBackend};
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::Block as BlockT;
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use thiserror::Error;

#[cfg(any(test, feature = "aura-integration"))]
pub mod aura;

#[cfg(any(test, feature = "bioauth-pallet-integration"))]
pub mod bioauth;

#[cfg(test)]
mod tests;

mod traits;

pub use traits::*;

/// A block-import handler for Bioauth.
pub struct BioauthBlockImport<Backend, Block: BlockT, Client, BAX, AV> {
    /// The client to interact with the chain.
    inner: Arc<Client>,
    /// The block author extractor.
    block_author_extractor: BAX,
    /// The bioauth auhtrization verifier.
    authorization_verifier: AV,
    /// A phantom data for Backend.
    _phantom_back_end: PhantomData<Backend>,
    /// A phantom data for Block.
    _phantom_block: PhantomData<Block>,
}

/// BioauthBlockImport Error Type.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum BioauthBlockImportError<BAX, AV>
where
    BAX: std::error::Error,
    AV: std::error::Error,
{
    /// The block author isn't Bioauth authorized.
    #[error("the block author isn't bioauth-authorized")]
    NotBioauthAuthorized,
    /// Block author extraction error.
    #[error("unable to extract block author: {0}")]
    BlockAuthorExtraction(BAX),
    /// Authorization verification error.
    #[error("unable verify the authorization: {0}")]
    AuthorizationVerifier(AV),
}

impl<BE, Block: BlockT, Client, BAX, AV> BioauthBlockImport<BE, Block, Client, BAX, AV> {
    /// Simple constructor.
    pub fn new(inner: Arc<Client>, block_author_extractor: BAX, authorization_verifier: AV) -> Self
    where
        BE: Backend<Block> + 'static,
    {
        Self {
            inner,
            block_author_extractor,
            authorization_verifier,
            _phantom_back_end: PhantomData,
            _phantom_block: PhantomData,
        }
    }
}

impl<BE, Block: BlockT, Client, BAX, AV> Clone for BioauthBlockImport<BE, Block, Client, BAX, AV>
where
    BAX: Clone,
    AV: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            block_author_extractor: self.block_author_extractor.clone(),
            authorization_verifier: self.authorization_verifier.clone(),
            _phantom_back_end: PhantomData,
            _phantom_block: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<BE, Block: BlockT, Client, BAX: Clone, AV: Clone> BlockImport<Block>
    for BioauthBlockImport<BE, Block, Client, BAX, AV>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + Finalizer<Block, BE>,
    for<'a> &'a Client:
        BlockImport<Block, Error = ConsensusError, Transaction = TransactionFor<Client, Block>>,
    TransactionFor<Client, Block>: 'static,
    BAX: BlockAuthorExtractor<Block = Block> + Send,
    AV: AuthorizationVerifier<Block = Block, PublicKeyType = BAX::PublicKeyType> + Send,
    <BAX as BlockAuthorExtractor>::PublicKeyType: Send + Sync,
    <BAX as BlockAuthorExtractor>::Error: std::error::Error + Send + Sync + 'static,
    <AV as AuthorizationVerifier>::Error: std::error::Error + Send + Sync + 'static,
    BE: Backend<Block>,
{
    type Error = ConsensusError;

    type Transaction = TransactionFor<Client, Block>;

    /// Check block preconditions. Only entire structure of a block.
    async fn check_block(
        &mut self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    /// Import a block.
    /// Cached data can be accessed through the blockchain cache.
    async fn import_block(
        &mut self,
        block: BlockImportParams<Block, Self::Transaction>,
        cache: HashMap<well_known_cache_keys::Id, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        // Extract a number of the last imported block.
        let at = sp_api::BlockId::Hash(self.inner.info().best_hash);

        let mkerr = |err: BioauthBlockImportError<BAX::Error, AV::Error>| -> ConsensusError {
            ConsensusError::Other(Box::new(err))
        };

        let author_public_key = self
            .block_author_extractor
            .extract_block_author(&at, &block.header)
            .map_err(|err| mkerr(BioauthBlockImportError::BlockAuthorExtraction(err)))?;

        let is_authorized = self
            .authorization_verifier
            .is_authorized(&at, &author_public_key)
            .map_err(|err| mkerr(BioauthBlockImportError::AuthorizationVerifier(err)))?;

        if !is_authorized {
            return Err(mkerr(BioauthBlockImportError::NotBioauthAuthorized));
        }

        // Finalize previous imported block.
        self.inner
            .finalize_block(at, None, false)
            .map_err(|_| sp_consensus::Error::CannotPropose)?;

        // Import a new block.
        self.inner.import_block(block, cache).await
    }
}
