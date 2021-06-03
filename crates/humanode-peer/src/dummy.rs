//! Some dummy implementations that we use just for now to make things work.

use sp_consensus::import_queue::{CacheKeyId, Verifier};
use sp_consensus::{BlockImportParams, BlockOrigin};
use sp_runtime::traits::Block as BlockT;
use sp_runtime::Justifications;

/// A verifier for Dummy blocks.
#[derive(Default, Debug)]
pub struct DummyVerifier;

#[async_trait::async_trait]
impl<B: BlockT> Verifier<B> for DummyVerifier {
    async fn verify(
        &mut self,
        origin: BlockOrigin,
        header: B::Header,
        _justifications: Option<Justifications>,
        _body: Option<Vec<B::Extrinsic>>,
    ) -> Result<(BlockImportParams<B, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
        // TODO: implement verify logic

        let import_block = BlockImportParams::new(origin, header);

        Ok((import_block, None))
    }
}
