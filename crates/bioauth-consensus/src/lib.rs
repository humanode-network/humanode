//! A block-import handler for Bioauth.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use pallet_bioauth::BioauthApi;
use sc_client_api::{backend::Backend, Finalizer};
use sp_api::{Decode, ProvideRuntimeApi, TransactionFor};
use sp_blockchain::HeaderBackend;
use sp_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, Error as ConsensusError, ImportResult,
};
use sp_consensus_aura::{AuraApi, Slot};
use sp_runtime::generic::OpaqueDigestItemId;
use sp_runtime::traits::{Block as BlockT, Header};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use thiserror::Error;

#[cfg(test)]
mod tests;

/// A block-import handler for Bioauth.
pub struct BioauthBlockImport<Backend, Block: BlockT, Client> {
    /// The client to interact with the chain.
    inner: Arc<Client>,
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

impl<Backend, Block: BlockT, Client> Clone for BioauthBlockImport<Backend, Block, Client> {
    fn clone(&self) -> Self {
        BioauthBlockImport {
            inner: Arc::clone(&self.inner),
            _phantom_back_end: PhantomData,
            _phantom_block: PhantomData,
        }
    }
}

impl<BE, Block: BlockT, Client> BioauthBlockImport<BE, Block, Client> {
    /// Simple constructor.
    pub fn new(inner: Arc<Client>) -> Self
    where
        BE: Backend<Block> + 'static,
    {
        BioauthBlockImport {
            inner,
            _phantom_back_end: PhantomData,
            _phantom_block: PhantomData,
        }
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
        cache: HashMap<sp_consensus::import_queue::CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        // Extract a number of the last imported block.
        let at = &sp_api::BlockId::Hash(self.inner.info().best_hash);

        // Extract current valid Aura authorities list.
        let authorities = self.inner.runtime_api().authorities(at).map_err(|_| {
            sp_consensus::Error::Other(Box::new(BioauthBlockImportError::ErrorExtractAuthorities))
        })?;

        // Extract current slot of a new produced block.
        let mut slot = block
            .header
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
        let author = match authorities.get(author_index as usize).cloned() {
            Some(v) => v.to_string().as_bytes().to_vec(),
            None => {
                return Err(sp_consensus::Error::Other(Box::new(
                    BioauthBlockImportError::InvalidSlotNumber,
                )))
            }
        };

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

        let is_authorized = stored_tickets.iter().any(|x| x.public_key == author);

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
