//! RPC subsystem instantiation logic.

use std::sync::Arc;

use bioauth_flow::rpc::{Bioauth, BioauthApi, LivenessDataTxSlot, ValidatorKeyExtractorT};
use fc_rpc::{
    EthApi, EthApiServer, EthPubSubApi, EthPubSubApiServer, HexEncodedIdProvider, Web3Api,
    Web3ApiServer,
};
use fc_rpc::{
    EthBlockDataCache, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
    SchemaV2Override, StorageOverride,
};
use humanode_runtime::{opaque::Block, AccountId, Balance, Hash, Index, UnixMilliseconds};
use jsonrpc_pubsub::manager::SubscriptionManager;
use pallet_ethereum::EthereumStorageSchema;
use sc_client_api::{
    backend::{AuxStore, Backend, StorageProvider},
    client::BlockchainEvents,
};
use sc_network::NetworkService;
use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{Encode, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use std::collections::BTreeMap;

/// RPC subsystem dependencies.
pub struct Deps<C, P, VKE, A: ChainApi> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls.
    pub deny_unsafe: DenyUnsafe,
    /// An ready robonode API client to tunnel the calls to.
    pub robonode_client: Arc<robonode_client::Client>,
    /// The liveness data tx slot to use in the bioauth flow RPC.
    pub bioauth_flow_slot: Arc<LivenessDataTxSlot>,
    /// Extracts the currently used validator key.
    pub validator_key_extractor: VKE,
    /// Graph pool instance.
    pub graph: Arc<Pool<A>>,
    /// Network service
    pub network: Arc<NetworkService<Block, Hash>>,
    /// Backend.
    pub backend: Arc<fc_db::Backend<Block>>,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
}

/// Instantiate all RPC extensions.
pub fn create<C, P, BE, VKE, A>(
    deps: Deps<C, P, VKE, A>,
    subscription_task_executor: SubscriptionTaskExecutor,
) -> jsonrpc_core::IoHandler<sc_rpc_api::Metadata>
where
    BE: Backend<Block> + 'static,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: bioauth_flow_api::BioauthFlowApi<Block, VKE::PublicKeyType, UnixMilliseconds>,
    C::Api: BlockBuilder<Block>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    VKE: ValidatorKeyExtractorT + Send + Sync + 'static,
    VKE::PublicKeyType: Encode,
    VKE::Error: std::fmt::Debug,
    P: TransactionPool<Block = Block> + 'static,
    A: ChainApi<Block = Block> + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let Deps {
        client,
        pool,
        deny_unsafe,
        robonode_client,
        bioauth_flow_slot,
        validator_key_extractor,
        graph,
        network,
        backend,
        max_past_logs,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        Arc::clone(&client),
        Arc::clone(&pool),
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        Arc::clone(&client),
    )));

    io.extend_with(BioauthApi::to_delegate(Bioauth::new(
        robonode_client,
        bioauth_flow_slot,
        validator_key_extractor,
        Arc::clone(&client),
    )));

    let mut overrides_map = BTreeMap::new();
    overrides_map.insert(
        EthereumStorageSchema::V1,
        Box::new(SchemaV1Override::new(Arc::clone(&client)))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );
    overrides_map.insert(
        EthereumStorageSchema::V2,
        Box::new(SchemaV2Override::new(Arc::clone(&client)))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );

    let overrides = Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::new(Arc::clone(&client))),
    });

    let block_data_cache = Arc::new(EthBlockDataCache::new(50, 50));

    io.extend_with(EthApiServer::to_delegate(EthApi::new(
        Arc::clone(&client),
        Arc::clone(&pool),
        graph,
        humanode_runtime::TransactionConverter,
        Arc::clone(&network),
        Vec::new(),
        Arc::clone(&overrides),
        Arc::clone(&backend),
        true,
        max_past_logs,
        Arc::clone(&block_data_cache),
    )));

    io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(Arc::clone(
        &client,
    ))));

    io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
        pool,
        client,
        network,
        SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
            HexEncodedIdProvider::default(),
            Arc::new(subscription_task_executor),
        ),
        overrides,
    )));

    io
}
