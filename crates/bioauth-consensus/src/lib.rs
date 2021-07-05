//! A block-import handler for Bioauth.

use sp_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, Error as ConsensusError, ImportResult,
    JustificationImport,
};
use sp_runtime::traits::{Block as BlockT, NumberFor};
use sp_runtime::Justification;
use std::{collections::HashMap, marker::PhantomData};

/// A block-import handler for Bioauth.
pub struct BioauthBlockImport<Block: BlockT> {
    _phantom: PhantomData<Block>,
}

impl<Block: BlockT> Clone for BioauthBlockImport<Block> {
    fn clone(&self) -> Self {
        BioauthBlockImport {
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<Block: BlockT> JustificationImport<Block> for BioauthBlockImport<Block> {
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
impl<Block: BlockT> BlockImport<Block> for BioauthBlockImport<Block> {
    type Error = ConsensusError;

    type Transaction = ();

    async fn check_block(
        &mut self,
        _block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        todo!()
    }

    async fn import_block(
        &mut self,
        _block: BlockImportParams<Block, Self::Transaction>,
        _cache: HashMap<sp_consensus::import_queue::CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        todo!()
    }
}
