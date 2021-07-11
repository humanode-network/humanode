//! A block-import handler for Bioauth.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use pallet_bioauth::BioauthAPI;
use sp_api::{Decode, ProvideRuntimeApi, TransactionFor};
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, Error as ConsensusError, ImportResult,
    JustificationImport,
};
use sp_consensus_aura::{AuraApi, Slot};
use sp_runtime::generic::OpaqueDigestItemId;
use sp_runtime::traits::{Block as BlockT, Header, NumberFor};
use sp_runtime::Justification;
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use thiserror::Error;

/// A block-import handler for Bioauth.
pub struct BioauthBlockImport<Block: BlockT, Client> {
    /// The client to interact with the chain.
    inner: Arc<Client>,
    /// A standart field that is defined at BlockImport logic.
    _phantom: PhantomData<Block>,
}

/// BioauthBlockImport Error Type.
#[derive(Error, Debug)]
pub enum BioauthBlockImportError {
    /// Block Author isn't Bioauth authorised
    #[error("Block Author isn't Bioauth authorised")]
    NotBioauthAuthorised,
    /// Invalid  slot number.
    #[error("Invalid slot number")]
    InvalidSlotNumber,
    /// Invalid block author.
    #[error("Invalid block author")]
    InvalidBlockAuthor,
    /// Error with extracting current stored auth tickets
    #[error("Can't get current stored auth tickets")]
    ErrorExtractStoredAuthTickets,
}

impl<Block: BlockT, Client> Clone for BioauthBlockImport<Block, Client> {
    fn clone(&self) -> Self {
        BioauthBlockImport {
            inner: Arc::<Client>::clone(&self.inner),
            _phantom: PhantomData,
        }
    }
}

impl<Block: BlockT, Client> BioauthBlockImport<Block, Client> {
    /// Simple constructor
    pub fn new(inner: Arc<Client>) -> Self {
        BioauthBlockImport {
            inner,
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<Block: BlockT, Client> JustificationImport<Block> for BioauthBlockImport<Block, Client> {
    type Error = ConsensusError;

    /// Import a Block justification and finalize the given block.
    fn import_justification(
        &mut self,
        _hash: Block::Hash,
        _number: NumberFor<Block>,
        _justification: Justification,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

#[async_trait::async_trait]
impl<Block: BlockT, Client> BlockImport<Block> for BioauthBlockImport<Block, Client>
where
    Client: HeaderBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + HeaderBackend<Block>
        + ProvideRuntimeApi<Block>
        + BlockImport<Block, Transaction = TransactionFor<Client, Block>, Error = sp_consensus::Error>
        + Send
        + Sync,
    for<'a> &'a Client:
        BlockImport<Block, Error = ConsensusError, Transaction = TransactionFor<Client, Block>>,
    TransactionFor<Client, Block>: 'static,
    Client::Api: AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
    Client::Api: BioauthAPI<Block>,
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
        // Extract a number of the last imported block
        let at = &sp_api::BlockId::Hash(self.inner.info().best_hash);

        // Extract current valid Aura authorities list
        let authorities = self.inner.runtime_api().authorities(at).ok().unwrap();

        // Extract current slot of a new produced block
        let mut slot = match block
            .header
            .digest()
            .log(|l| l.try_as_raw(OpaqueDigestItemId::PreRuntime(b"aura")))
        {
            Some(v) => v,
            None => {
                return Err(sp_consensus::Error::Other(Box::new(
                    BioauthBlockImportError::InvalidSlotNumber,
                )))
            }
        };

        // Decode slot number
        let slot_decoded = match Slot::decode(&mut slot) {
            Ok(v) => v,
            Err(_e) => {
                return Err(sp_consensus::Error::Other(Box::new(
                    BioauthBlockImportError::InvalidSlotNumber,
                )))
            }
        };

        // Get Author index of a new proposed block
        let author_index = *slot_decoded % authorities.len() as u64;

        // Determine an Author of a new proposed block
        let author = match authorities.get(author_index as usize).cloned() {
            Some(v) => v.to_string().as_bytes().to_vec(),
            None => {
                return Err(sp_consensus::Error::Other(Box::new(
                    BioauthBlockImportError::InvalidBlockAuthor,
                )))
            }
        };

        // Get current stored tickets
        let stored_tickets = match self.inner.runtime_api().get_stored_tickets(at) {
            Ok(v) => v,
            Err(_e) => {
                return Err(sp_consensus::Error::Other(Box::new(
                    BioauthBlockImportError::ErrorExtractStoredAuthTickets,
                )))
            }
        };

        let mut is_authorized = false;

        for existing in stored_tickets.iter() {
            if existing.public_key == author {
                is_authorized = true;
            }
        }

        if is_authorized {
            self.inner.import_block(block, cache).await
        } else {
            return Err(sp_consensus::Error::Other(Box::new(
                BioauthBlockImportError::NotBioauthAuthorised,
            )));
        }
    }
}
