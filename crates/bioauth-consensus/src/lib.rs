//! A block-import handler for Bioauth.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use author_validation::{AuthorizationVerifier, BlockAuthorExtractor};
use pallet_bioauth::BioauthApi;
use sc_client_api::{backend::Backend, Finalizer};
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{Decode, ProvideRuntimeApi, TransactionFor};
use sp_application_crypto::Public;
use sp_blockchain::{well_known_cache_keys, HeaderBackend};
use sp_consensus::{CanAuthorWith, Error as ConsensusError};
use sp_consensus_aura::{AuraApi, Slot};
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::generic::OpaqueDigestItemId;
use sp_runtime::traits::{Block as BlockT, Header};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use thiserror::Error;

mod author_validation;

#[cfg(test)]
mod tests;

/// A block-import handler for Bioauth.
pub struct BioauthBlockImport<Backend, Block: BlockT, Client> {
    /// The client to interact with the chain.
    inner: Arc<Client>,
    /// Keystore to extract validator public key.
    keystore: SyncCryptoStorePtr,
    /// A phantom data for Backend.
    _phantom_back_end: PhantomData<Backend>,
    /// A phantom data for Block.
    _phantom_block: PhantomData<Block>,
}

/// BioauthBlockImport Error Type.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum BioauthBlockImportError {
    /// Block Author isn't Bioauth authorized.
    #[error("Block Author isn't bioauth-authorized")]
    NotBioauthAuthorized,
    /// Invalid  slot number.
    #[error("Invalid slot number")]
    InvalidSlotNumber,
    /// Error with extracting current stored auth tickets.
    #[error("Can't get current stored auth tickets")]
    ErrorExtractStoredAuthTickets,
    /// Error with extracting current authorities list.
    #[error("Can't get current authorities list")]
    ErrorExtractAuthorities,
}

impl<BE, Block: BlockT, Client> BioauthBlockImport<BE, Block, Client> {
    /// Simple constructor.
    pub fn new(inner: Arc<Client>, keystore: SyncCryptoStorePtr) -> Self
    where
        BE: Backend<Block> + 'static,
    {
        BioauthBlockImport {
            inner,
            keystore,
            _phantom_back_end: PhantomData,
            _phantom_block: PhantomData,
        }
    }
}

impl<Backend, Block: BlockT, Client> Clone for BioauthBlockImport<Backend, Block, Client> {
    fn clone(&self) -> Self {
        BioauthBlockImport {
            inner: Arc::clone(&self.inner),
            keystore: Arc::clone(&self.keystore),
            _phantom_back_end: PhantomData,
            _phantom_block: PhantomData,
        }
    }
}

impl<Backend, Block: BlockT, Client> BlockAuthorExtractor
    for BioauthBlockImport<Backend, Block, Client>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    Client::Api: AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
{
    type Error = ConsensusError;
    type BlockHeader = Block::Header;
    type PublicKeyType = Vec<u8>;

    fn extract_block_author(
        &self,
        block_header: &Self::BlockHeader,
    ) -> Result<Vec<u8>, Self::Error> {
        // Extract a number of the last imported block.
        let at = &sp_api::BlockId::Hash(self.inner.info().best_hash);

        // Extract current valid Aura authorities list.
        let authorities = self.inner.runtime_api().authorities(at).map_err(|_| {
            sp_consensus::Error::Other(Box::new(BioauthBlockImportError::ErrorExtractAuthorities))
        })?;

        // Extract current slot of a new produced block.
        let mut slot = block_header
            .digest()
            .log(|l| l.try_as_raw(OpaqueDigestItemId::PreRuntime(b"aura")))
            .ok_or_else(|| {
                sp_consensus::Error::Other(Box::new(BioauthBlockImportError::InvalidSlotNumber))
            })?;

        // Decode slot number.
        let slot_decoded = Slot::decode(&mut slot).map_err(|_| {
            sp_consensus::Error::Other(Box::new(BioauthBlockImportError::InvalidSlotNumber))
        })?;

        // Get Author index of a new proposed block.
        let author_index = *slot_decoded % authorities.len() as u64;

        // Determine an Author of a new proposed block.
        let author_public_key =
            authorities
                .get(author_index as usize)
                .cloned()
                .ok_or_else(|| {
                    sp_consensus::Error::Other(Box::new(BioauthBlockImportError::InvalidSlotNumber))
                })?;

        Ok(author_public_key.to_raw_vec())
    }
}

impl<Backend, Block: BlockT, Client> AuthorizationVerifier
    for BioauthBlockImport<Backend, Block, Client>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    Client::Api: BioauthApi<Block>,
{
    type Error = ConsensusError;
    type PublicKeyType = [u8];

    fn is_authorized(&self, author_public_key: &Self::PublicKeyType) -> Result<bool, Self::Error> {
        // Extract a number of the last imported block.
        let at = &sp_api::BlockId::Hash(self.inner.info().best_hash);

        // Get current stored tickets.
        let stored_tickets = self
            .inner
            .runtime_api()
            .stored_auth_tickets(at)
            .map_err(|_| {
                sp_consensus::Error::Other(Box::new(
                    BioauthBlockImportError::ErrorExtractStoredAuthTickets,
                ))
            })?;

        let is_authorized = stored_tickets
            .iter()
            .any(|ticket| ticket.public_key == author_public_key);

        Ok(is_authorized)
    }
}

#[async_trait::async_trait]
impl<BE, Block: BlockT, Client> BlockImport<Block> for BioauthBlockImport<BE, Block, Client>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + Finalizer<Block, BE>,
    for<'a> &'a Client:
        BlockImport<Block, Error = ConsensusError, Transaction = TransactionFor<Client, Block>>,
    TransactionFor<Client, Block>: 'static,
    Client::Api: AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
    Client::Api: BioauthApi<Block>,
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
        let at = &sp_api::BlockId::Hash(self.inner.info().best_hash);

        let author_public_key = self.extract_block_author(&block.header)?;

        let is_authorized = self.is_authorized(author_public_key.as_slice())?;

        if !is_authorized {
            return Err(sp_consensus::Error::Other(Box::new(
                BioauthBlockImportError::NotBioauthAuthorized,
            )));
        }

        // Finalize previous imported block.
        self.inner
            .finalize_block(*at, None, false)
            .map_err(|_| sp_consensus::Error::CannotPropose)?;

        // Import a new block.
        self.inner.import_block(block, cache).await
    }
}

impl<BE, Block: BlockT, Client> CanAuthorWith<Block> for BioauthBlockImport<BE, Block, Client>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    Client::Api: BioauthApi<Block>,
{
    fn can_author_with(&self, _at: &sp_api::BlockId<Block>) -> Result<(), String> {
        let mut aura_public_keys = sp_keystore::SyncCryptoStore::sr25519_public_keys(
            self.keystore.as_ref(),
            sp_application_crypto::key_types::AURA,
        );
        assert_eq!(aura_public_keys.len(), 1);
        let aura_public_key = match aura_public_keys.drain(..).next() {
            Some(v) => v,
            _ => return Err("You aren't Aura validator.".to_string()),
        };

        let is_authorized = self
            .is_authorized(aura_public_key.as_slice())
            .map_err(|e| e.to_string())?;

        if !is_authorized {
            return Err("You aren't bioauth-authorized.".to_string());
        }

        Ok(())
    }
}
