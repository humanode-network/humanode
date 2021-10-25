//! A block-import handler for Bioauth.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use futures::future;
use futures::future::FutureExt;
use sc_client_api::backend::Backend;
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{HeaderT, ProvideRuntimeApi, TransactionFor};
use sp_blockchain::{well_known_cache_keys, HeaderBackend};
use sp_consensus::{Environment, Error as ConsensusError};
use sp_runtime::traits::Block as BlockT;
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use thiserror::Error;

#[cfg(feature = "api-integration")]
pub mod api;

#[cfg(feature = "keystore-integration")]
pub mod keystore;

#[cfg(feature = "aura-integration")]
pub mod aura;

#[cfg(test)]
mod tests;

mod traits;

pub use traits::*;

/// A block-import handler for Bioauth.
pub struct BioauthBlockImport<Backend, Block: BlockT, Client, BI, BAX, AV> {
    /// The client to interact with the chain.
    client: Arc<Client>,
    /// The inner block import to wrap.
    inner: BI,
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

impl<BE, Block: BlockT, Client, BI, BAX, AV> BioauthBlockImport<BE, Block, Client, BI, BAX, AV> {
    /// Simple constructor.
    pub fn new(
        client: Arc<Client>,
        inner: BI,
        block_author_extractor: BAX,
        authorization_verifier: AV,
    ) -> Self
    where
        BE: Backend<Block> + 'static,
    {
        Self {
            client,
            inner,
            block_author_extractor,
            authorization_verifier,
            _phantom_back_end: PhantomData,
            _phantom_block: PhantomData,
        }
    }
}

impl<BE, Block: BlockT, Client, BI, BAX, AV> Clone
    for BioauthBlockImport<BE, Block, Client, BI, BAX, AV>
where
    BI: Clone,
    BAX: Clone,
    AV: Clone,
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            inner: self.inner.clone(),
            block_author_extractor: self.block_author_extractor.clone(),
            authorization_verifier: self.authorization_verifier.clone(),
            _phantom_back_end: PhantomData,
            _phantom_block: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<BE, Block: BlockT, Client, BI, BAX: Clone, AV: Clone> BlockImport<Block>
    for BioauthBlockImport<BE, Block, Client, BI, BAX, AV>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync,
    TransactionFor<Client, Block>: 'static,
    BI: BlockImport<Block, Error = ConsensusError, Transaction = TransactionFor<Client, Block>>
        + Send
        + Sync,
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
        let at = sp_api::BlockId::Hash(self.client.info().best_hash);

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

        // Import a new block and apply finality with Grandpa.
        self.inner.import_block(block, cache).await
    }
}

/// A Proposer handler for Bioauth.
pub struct BioauthProposer<Block: BlockT, BAP, VKE, AV> {
    /// A basic authorship proposer.
    base_proposer: BAP,
    /// Keystore to extract validator public key.
    validator_key_extractor: VKE,
    /// The bioauth auhtrization verifier.
    authorization_verifier: AV,
    /// A phantom data for Block.
    _phantom_block: PhantomData<Block>,
}

/// BioauthProposer Error Type.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum BioauthProposerError<VKE, AV>
where
    VKE: std::error::Error,
    AV: std::error::Error,
{
    /// Unable to find validator key for the node, no we aren't allow to produce blocks as if
    /// we are not bioauth-authorized.
    #[error("unable to extract validator key for this node, this node is not bioauth-authorized")]
    UnableToExtractValidatorKey,
    /// The block author isn't bioauth-authorized.
    #[error("the block author isn't bioauth-authorized")]
    NotBioauthAuthorized,
    /// Validator key extraction error.
    #[error("unable extract validator key: {0}")]
    ValidatorKeyExtraction(VKE),
    /// Authorization verification error.
    #[error("unable verify the authorization: {0}")]
    AuthorizationVerification(AV),
}

impl<Block: BlockT, BAP, VKE, AV> BioauthProposer<Block, BAP, VKE, AV> {
    /// Create a new [`BioauthProposer`].
    pub fn new(
        base_proposer: BAP,
        validator_key_extractor: VKE,
        authorization_verifier: AV,
    ) -> Self {
        Self {
            base_proposer,
            validator_key_extractor,
            authorization_verifier,
            _phantom_block: PhantomData,
        }
    }
}

impl<Block: BlockT, BAP, VKE, AV> Environment<Block> for BioauthProposer<Block, BAP, VKE, AV>
where
    BAP: Environment<Block> + Send + Sync + 'static,
    BAP::Error: Send,
    BAP::Proposer: Send,
    VKE: ValidatorKeyExtractor,
    <VKE as ValidatorKeyExtractor>::Error: std::error::Error + Send + Sync + 'static,
    AV: AuthorizationVerifier<Block = Block, PublicKeyType = VKE::PublicKeyType> + Send,
    <AV as AuthorizationVerifier>::Error: std::error::Error + Send + Sync + 'static,
{
    type Proposer = BAP::Proposer;

    type CreateProposer = future::BoxFuture<'static, Result<Self::Proposer, Self::Error>>;

    type Error = BAP::Error;

    fn init(&mut self, parent_header: &Block::Header) -> Self::CreateProposer {
        let mkerr = |err: BioauthProposerError<VKE::Error, AV::Error>| -> Self::CreateProposer {
            Box::pin(future::err(Self::Error::from(sp_consensus::Error::Other(
                Box::new(err),
            ))))
        };

        let validator_key = match self
            .validator_key_extractor
            .extract_validator_key()
            .map_err(|err| mkerr(BioauthProposerError::ValidatorKeyExtraction(err)))
        {
            Ok(v) => v,
            Err(err) => return err,
        };

        let validator_key = match validator_key
            .ok_or_else(|| mkerr(BioauthProposerError::UnableToExtractValidatorKey))
        {
            Ok(v) => v,
            Err(err) => return err,
        };

        let parent_hash = parent_header.hash();
        let at = sp_api::BlockId::hash(parent_hash);

        let is_authorized = match self
            .authorization_verifier
            .is_authorized(&at, &validator_key)
            .map_err(|err| mkerr(BioauthProposerError::AuthorizationVerification(err)))
        {
            Ok(v) => v,
            Err(err) => return err,
        };

        if !is_authorized {
            return mkerr(BioauthProposerError::NotBioauthAuthorized);
        }

        self.base_proposer.init(parent_header).boxed()
    }
}
