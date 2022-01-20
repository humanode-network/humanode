//! RPC subsystem instantiation logic.

use std::{collections::BTreeMap, sync::Arc};

use bioauth_flow::rpc::{Bioauth, BioauthApi, LivenessDataTxSlot, ValidatorKeyExtractorT};
use fc_rpc::{
    EthApi, EthApiServer, EthFilterApi, EthFilterApiServer, EthPubSubApi, EthPubSubApiServer,
    HexEncodedIdProvider, NetApi, NetApiServer, Web3Api, Web3ApiServer,
};
use fc_rpc::{
    EthBlockDataCache, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
    SchemaV2Override, SchemaV3Override, StorageOverride,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use humanode_runtime::{
    opaque::{Block, UncheckedExtrinsic},
    AccountId, Balance, Hash, Index, UnixMilliseconds,
};
use jsonrpc_pubsub::manager::SubscriptionManager;
use pallet_ethereum::EthereumStorageSchema;
use sc_client_api::{
    backend::{AuxStore, Backend, StateBackend, StorageProvider},
    client::BlockchainEvents,
};
use sc_network::NetworkService;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{Encode, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::BlakeTwo256;

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
    /// EthFilterApi pool.
    pub filter_pool: Option<FilterPool>,
    /// Maximum number of stored filters.
    pub max_stored_filters: usize,
    /// Backend.
    pub backend: Arc<fc_db::Backend<Block>>,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
    /// Maximum fee history cache size.
    pub fee_history_limit: u64,
    /// Fee history cache.
    pub fee_history_cache: FeeHistoryCache,
    /// Subscription task executor instance.
    pub subscription_task_executor: Arc<sc_rpc::SubscriptionTaskExecutor>,
    /// Ethereum data access overrides.
    pub overrides: Arc<OverrideHandle<Block>>,
    /// Cache for Ethereum block data.
    pub block_data_cache: Arc<EthBlockDataCache<Block>>,
}

/// A helper function to handle overrides.
pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
{
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
    overrides_map.insert(
        EthereumStorageSchema::V3,
        Box::new(SchemaV3Override::new(Arc::clone(&client)))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );

    Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::new(Arc::clone(&client))),
    })
}

/// Instantiate all RPC extensions.
pub fn create<C, P, BE, VKE, A>(
    deps: Deps<C, P, VKE, A>,
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
    C::Api: frontier_api::TransactionConverterApi<Block, UncheckedExtrinsic>,
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
        filter_pool,
        max_stored_filters,
        backend,
        max_past_logs,
        fee_history_limit,
        fee_history_cache,
        subscription_task_executor,
        overrides,
        block_data_cache,
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

    io.extend_with(EthApiServer::to_delegate(EthApi::new(
        Arc::clone(&client),
        Arc::clone(&pool),
        graph,
        primitives_frontier::RuntimeTransactionConverter::new(Arc::clone(&client)),
        Arc::clone(&network),
        Vec::new(),
        Arc::clone(&overrides),
        Arc::clone(&backend),
        true,
        max_past_logs,
        Arc::clone(&block_data_cache),
        fee_history_limit,
        fee_history_cache,
    )));

    io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(Arc::clone(
        &client,
    ))));

    io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
        pool,
        Arc::clone(&client),
        Arc::clone(&network),
        SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
            HexEncodedIdProvider::default(),
            subscription_task_executor,
        ),
        Arc::clone(&overrides),
    )));

    io.extend_with(NetApiServer::to_delegate(NetApi::new(
        Arc::clone(&client),
        Arc::clone(&network),
        // Whether to format the `peer_count` response as Hex (default) or not.
        true,
    )));

    if let Some(filter_pool) = filter_pool {
        io.extend_with(EthFilterApiServer::to_delegate(EthFilterApi::new(
            client,
            backend,
            filter_pool,
            max_stored_filters,
            max_past_logs,
            block_data_cache,
        )));
    }

    io
}
