use codec::{Decode, Encode};
use sc_client_api::{backend::AuxStore, BlockOf, UsageProvider};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{well_known_cache_keys::Id as CacheKeyId, HeaderBackend, ProvideCache};
use sp_consensus::{
    import_queue::{BasicQueue, BoxJustificationImport, DefaultImportQueue, Verifier},
    BlockCheckParams, BlockImport, BlockImportParams, BlockOrigin, Error as ConsensusError,
    ImportResult,
};
use sp_core::crypto::Pair;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::Justifications;
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData, sync::Arc};
use substrate_prometheus_endpoint::Registry;

/// Key pair definition app_sr.
mod app_sr25519 {
    use sp_application_crypto::{app_crypto, key_types::DUMMY, sr25519};
    app_crypto!(sr25519, DUMMY);
}

/// Dummy key pair is used for Dummy Consensus.
pub type DummyPair = app_sr25519::Pair;

/// A verifier for Dummy blocks.
#[derive(Default, Debug)]
pub struct DummyVerifier;

#[async_trait::async_trait]
impl<B: BlockT> Verifier<B> for DummyVerifier {
    async fn verify(
        &mut self,
        origin: BlockOrigin,
        header: B::Header,
        justifications: Option<Justifications>,
        mut body: Option<Vec<B::Extrinsic>>,
    ) -> Result<(BlockImportParams<B, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>), String> {
        // TODO: implement verify logic

        let import_block = BlockImportParams::new(origin, header);

        Ok((import_block, None))
    }
}

/// Parameters of [`import_queue`].
pub struct ImportQueueParams<'a, Block, I, C, S, CIDP> {
    /// The block import to use.
    pub block_import: I,
    /// The justification import.
    pub justification_import: Option<BoxJustificationImport<Block>>,
    /// The client to interact with the chain.
    pub client: Arc<C>,
    /// Something that can create the inherent data providers.
    pub create_inherent_data_providers: CIDP,
    /// The spawner to spawn background tasks.
    pub spawner: &'a S,
    /// The prometheus registry.
    pub registry: Option<&'a Registry>,
}

/// Start an import queue for the Dummy consensus algorithm.
pub fn import_queue<'a, P, Block, I, C, S, CIDP>(
    ImportQueueParams {
        block_import,
        justification_import,
        client,
        create_inherent_data_providers,
        spawner,
        registry,
    }: ImportQueueParams<'a, Block, I, C, S, CIDP>,
) -> Result<DefaultImportQueue<Block, C>, sp_consensus::Error>
where
    Block: BlockT,
    C: 'static
        + ProvideRuntimeApi<Block>
        + BlockOf
        + ProvideCache<Block>
        + Send
        + Sync
        + AuxStore
        + UsageProvider<Block>
        + HeaderBackend<Block>,
    I: BlockImport<Block, Error = ConsensusError, Transaction = sp_api::TransactionFor<C, Block>>
        + Send
        + Sync
        + 'static,
    P: Pair + Send + Sync + 'static,
    P::Public: Clone + Eq + Send + Sync + Hash + Debug + Encode + Decode,
    P::Signature: Encode + Decode,
    S: sp_core::traits::SpawnEssentialNamed,
    CIDP: CreateInherentDataProviders<Block, ()> + Sync + Send + 'static,
{
    let verifier = build_verifier::<P, _, _>(BuildVerifierParams {
        client,
        create_inherent_data_providers,
    });

    Ok(BasicQueue::new(
        verifier,
        Box::new(block_import),
        justification_import,
        spawner,
        registry,
    ))
}

/// A block-import handler for Dummy.
pub struct DummyBlockImport<Block: BlockT, C, I: BlockImport<Block>, P> {
    inner: I,
    client: Arc<C>,
    _phantom: PhantomData<(Block, P)>,
}

impl<Block: BlockT, C, I: Clone + BlockImport<Block>, P> Clone
    for DummyBlockImport<Block, C, I, P>
{
    fn clone(&self) -> Self {
        DummyBlockImport {
            inner: self.inner.clone(),
            client: self.client.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Block: BlockT, C, I: BlockImport<Block>, P> DummyBlockImport<Block, C, I, P> {
    /// New dummy block import.
    pub fn new(inner: I, client: Arc<C>) -> Self {
        Self {
            inner,
            client,
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<Block: BlockT, C, I, P> BlockImport<Block> for DummyBlockImport<Block, C, I, P>
where
    I: BlockImport<Block, Transaction = sp_api::TransactionFor<C, Block>> + Send + Sync,
    I::Error: Into<ConsensusError>,
    C: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    P: Pair + Send + Sync + 'static,
    P::Public: Clone + Eq + Send + Sync + Hash + Debug + Encode + Decode,
    P::Signature: Encode + Decode,
    sp_api::TransactionFor<C, Block>: Send + 'static,
{
    type Error = ConsensusError;
    type Transaction = sp_api::TransactionFor<C, Block>;

    async fn check_block(
        &mut self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await.map_err(Into::into)
    }

    async fn import_block(
        &mut self,
        block: BlockImportParams<Block, Self::Transaction>,
        new_cache: HashMap<CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        // TODO: implement required logic

        self.inner
            .import_block(block, new_cache)
            .await
            .map_err(Into::into)
    }
}
