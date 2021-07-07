//! A block-import handler for Bioauth.

use sp_api::{ProvideRuntimeApi, TransactionFor};
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, Error as ConsensusError, ImportResult,
    JustificationImport,
};
use sp_runtime::generic::OpaqueDigestItemId;
use sp_runtime::traits::{Block as BlockT, Header, NumberFor};
use sp_runtime::Justification;
use std::{collections::HashMap, marker::PhantomData, sync::Arc};

/// A block-import handler for Bioauth.
pub struct BioauthBlockImport<Block: BlockT, Client> {
    inner: Arc<Client>,
    _phantom: PhantomData<Block>,
}

impl<Block: BlockT, Client> Clone for BioauthBlockImport<Block, Client> {
    fn clone(&self) -> Self {
        BioauthBlockImport {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Block: BlockT, Client> BioauthBlockImport<Block, Client> {
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
{
    type Error = ConsensusError;

    type Transaction = TransactionFor<Client, Block>;

    async fn check_block(
        &mut self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    async fn import_block(
        &mut self,
        block: BlockImportParams<Block, Self::Transaction>,
        cache: HashMap<sp_consensus::import_queue::CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        let id = OpaqueDigestItemId::Consensus(&sp_consensus_aura::AURA_ENGINE_ID);

        let aura_authority_key = match block.header.digest().log(|l| l.try_as_raw(id)) {
            Some(key) => key,
            None => return Err(sp_consensus::Error::CannotPropose),
        };

        let list = pallet_bioauth::StoredAuthTickets::<humanode_runtime::Runtime>::get();
        let mut is_authorized = false;
        for existing in list.iter() {
            if existing.public_key == aura_authority_key.to_vec() {
                is_authorized = true;
                break;
            }
        }

        if is_authorized {
            self.inner.import_block(block, cache).await
        } else {
            Err(sp_consensus::Error::CannotPropose)
        }
    }
}
