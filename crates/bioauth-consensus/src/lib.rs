//! A block-import handler for Bioauth.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use futures::future;
use futures::future::FutureExt;
use sc_client_api::{backend::Backend, Finalizer};
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{HeaderT, ProvideRuntimeApi, TransactionFor};
use sp_blockchain::{well_known_cache_keys, HeaderBackend};
use sp_consensus::{Environment, Error as ConsensusError};
use sp_keystore::SyncCryptoStorePtr;
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

        // Import a new block.
        self.inner.import_block(block, cache).await
    }
}

/// A Proposer handler for Bioauth.
pub struct BioauthProposer<Block: BlockT, AV, BAP> {
    /// A basic authorship proposer.
    base_proposer: BAP,
    /// Keystore to extract validator public key.
    keystore: SyncCryptoStorePtr,
    /// The bioauth auhtrization verifier.
    authorization_verifier: AV,
    /// A phantom data for Block.
    _phantom_block: PhantomData<Block>,
}

/// BioauthProposer Error Type.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum BioauthProposerError<AV>
where
    AV: std::error::Error,
{
    /// The block author isn't Bioauth authorized.
    #[error("the block author isn't bioauth-authorized")]
    NotBioauthAuthorized,
    /// Authorization verification error.
    #[error("unable verify the authorization: {0}")]
    AuthorizationVerifier(AV),
}

impl<Block: BlockT, AV, BAP> BioauthProposer<Block, AV, BAP> {
    /// Simple constructor.
    pub fn new(
        base_proposer: BAP,
        keystore: SyncCryptoStorePtr,
        authorization_verifier: AV,
    ) -> Self {
        BioauthProposer {
            base_proposer,
            keystore,
            authorization_verifier,
            _phantom_block: PhantomData,
        }
    }
}

impl<Block: BlockT, AV, BAP> Environment<Block> for BioauthProposer<Block, AV, BAP>
where
    AV: AuthorizationVerifier<
            Block = Block,
            PublicKeyType = sp_consensus_aura::sr25519::AuthorityId,
        > + Send,
    <AV as AuthorizationVerifier>::Error: std::error::Error + Send + Sync + 'static,
    BAP: Environment<Block> + Send + Sync + 'static,
    BAP::Error: Send,
    BAP::Proposer: Send,
{
    type Proposer = BAP::Proposer;

    type CreateProposer = future::BoxFuture<'static, Result<Self::Proposer, Self::Error>>;

    type Error = BAP::Error;

    fn init(&mut self, parent_header: &Block::Header) -> Self::CreateProposer {
        let mkerr = |err: BioauthProposerError<AV::Error>| -> Self::CreateProposer {
            Box::pin(future::err(Self::Error::from(sp_consensus::Error::Other(
                Box::new(err),
            ))))
        };

        let keystore_ref = self.keystore.as_ref();

        let aura_public_keys = tokio::task::block_in_place(move || {
            sp_keystore::SyncCryptoStore::sr25519_public_keys(
                keystore_ref,
                sp_application_crypto::key_types::AURA,
            )
        });

        assert!(
            aura_public_keys.len() == 1,
            "The list of aura public keys should contain only 1 key, please report this"
        );

        let aura_public_key = aura_public_keys[0];

        let parent_hash = parent_header.hash();
        let at = sp_api::BlockId::hash(parent_hash);

        let is_authorized = match self
            .authorization_verifier
            .is_authorized(&at, &aura_public_key.into())
        {
            Ok(v) => v,
            Err(err) => return mkerr(BioauthProposerError::AuthorizationVerifier(err)),
        };

        if !is_authorized {
            return mkerr(BioauthProposerError::NotBioauthAuthorized);
        }

        self.base_proposer.init(parent_header).boxed()
    }
}
