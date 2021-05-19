use std::marker::PhantomData;

use sp_consensus::{Error as ConsensusError, SelectChain};
use sp_runtime::traits::Block as BlockT;

#[derive(Clone, Default)]
pub struct DummyConsensus<Block> {
    block_type: PhantomData<Block>,
}

impl<Block: BlockT> SelectChain<Block> for DummyConsensus<Block> {
    fn leaves(&self) -> Result<Vec<<Block as BlockT>::Hash>, ConsensusError> {
        todo!()
    }

    fn best_chain(&self) -> Result<<Block as BlockT>::Header, ConsensusError> {
        todo!()
    }
}
