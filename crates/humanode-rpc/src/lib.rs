//! RPC subsystem instantiation logic.

use std::{collections::BTreeMap, sync::Arc};

use author_ext_api::AuthorExtApi;
use author_ext_rpc::{AuthorExt, AuthorExtServer};
use bioauth_flow_rpc::{Bioauth, BioauthServer, Signer, SignerFactory};
use bioauth_keys::traits::KeyExtractor as KeyExtractorT;
use fc_rpc::{
    Eth, EthApiServer, EthBlockDataCacheTask, EthFilter, EthFilterApiServer, EthPubSub,
    EthPubSubApiServer, Net, NetApiServer, OverrideHandle, RuntimeApiStorageOverride,
    SchemaV1Override, SchemaV2Override, SchemaV3Override, StorageOverride, Web3, Web3ApiServer,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use fp_storage::EthereumStorageSchema;
use humanode_runtime::{
    opaque::Block, AccountId, Balance, BlockNumber, Hash, Index, UnixMilliseconds,
};
use jsonrpsee::RpcModule;
use sc_client_api::{
    backend::{AuxStore, Backend, StateBackend, StorageProvider},
    client::BlockchainEvents,
};
use sc_consensus_babe::{Config, Epoch};
use sc_consensus_babe_rpc::{Babe, BabeApiServer};
use sc_consensus_epochs::SharedEpochChanges;
use sc_finality_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_finality_grandpa_rpc::{Grandpa, GrandpaApiServer};
use sc_network::NetworkService;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{Encode, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::traits::BlakeTwo256;

/// Extra dependencies for AuthorExt.
pub struct AuthorExtDeps<VKE> {
    /// Extracts the currently used author validator key.
    pub author_validator_key_extractor: VKE,
}

/// Extra dependencies for Bioauth.
pub struct BioauthDeps<VKE, VSF> {
    /// An ready robonode API client to tunnel the calls to.
    pub robonode_client: Arc<robonode_client::Client>,
    /// Extracts the currently used bioauth validator key.
    pub bioauth_validator_key_extractor: VKE,
    /// A factory for making signers by the bioauth validator public keys.
    pub bioauth_validator_signer_factory: VSF,
}

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// BABE protocol config.
    pub babe_config: Config,
    /// BABE pending epoch changes.
    pub babe_shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
    /// The keystore that manages the keys of the node.
    pub keystore: SyncCryptoStorePtr,
}

/// Extra dependencies for GRANDPA.
pub struct GrandpaDeps<BE> {
    /// Voting round info.
    pub grandpa_shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub grandpa_shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub grandpa_justification_stream: GrandpaJustificationStream<Block>,
    /// Finality proof provider.
    pub grandpa_finality_provider: Arc<FinalityProofProvider<BE, Block>>,
}

/// Extra EVM related dependencies.
pub struct EvmDeps {
    /// EthFilterApi pool.
    pub eth_filter_pool: Option<FilterPool>,
    /// Maximum number of stored filters.
    pub eth_max_stored_filters: usize,
    /// Backend.
    pub eth_backend: Arc<fc_db::Backend<Block>>,
    /// Maximum number of logs in a query.
    pub eth_max_past_logs: u32,
    /// Maximum fee history cache size.
    pub eth_fee_history_limit: u64,
    /// Fee history cache.
    pub eth_fee_history_cache: FeeHistoryCache,
    /// Ethereum data access overrides.
    pub eth_overrides: Arc<OverrideHandle<Block>>,
    /// Cache for Ethereum block data.
    pub eth_block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
}

/// RPC subsystem dependencies.
pub struct Deps<C, P, BE, VKE, VSF, A: ChainApi, SC> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls.
    pub deny_unsafe: DenyUnsafe,
    /// Graph pool instance.
    pub graph: Arc<Pool<A>>,
    /// Network service
    pub network: Arc<NetworkService<Block, Hash>>,
    /// AuthorExt specific dependencies.
    pub author_ext: AuthorExtDeps<VKE>,
    /// Bioauth specific dependencies.
    pub bioauth: BioauthDeps<VKE, VSF>,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<BE>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// EVM specific dependencies.
    pub evm: EvmDeps,
    /// Subscription task executor instance.
    pub subscription_task_executor: sc_rpc::SubscriptionTaskExecutor,
}

/// A helper function to handle overrides.
pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
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
pub fn create<C, P, BE, VKE, VSF, A, SC>(
    deps: Deps<C, P, BE, VKE, VSF, A, SC>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    BE: Backend<Block> + 'static,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: bioauth_flow_api::BioauthFlowApi<Block, VKE::PublicKeyType, UnixMilliseconds>,
    C::Api: BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    C::Api: AuthorExtApi<Block, VKE::PublicKeyType>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
    VKE: KeyExtractorT + Send + Sync + 'static,
    VKE::PublicKeyType: Encode + AsRef<[u8]> + Send + Sync,
    VKE::Error: std::fmt::Debug,
    VSF: SignerFactory<Vec<u8>, VKE::PublicKeyType> + Send + Sync + 'static,
    VSF::Signer: Send + Sync + 'static,
    <<VSF as SignerFactory<Vec<u8>, VKE::PublicKeyType>>::Signer as Signer<Vec<u8>>>::Error:
        std::error::Error + 'static,
    A: ChainApi<Block = Block> + 'static,
    SC: SelectChain<Block> + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut io = RpcModule::new(());

    let Deps {
        client,
        pool,
        deny_unsafe,
        graph,
        network,
        author_ext,
        bioauth,
        babe,
        grandpa,
        select_chain,
        evm,
        subscription_task_executor,
    } = deps;

    let AuthorExtDeps {
        author_validator_key_extractor,
    } = author_ext;

    let BioauthDeps {
        robonode_client,
        bioauth_validator_key_extractor,
        bioauth_validator_signer_factory,
    } = bioauth;

    let BabeDeps {
        keystore,
        babe_config,
        babe_shared_epoch_changes,
    } = babe;

    let GrandpaDeps {
        grandpa_shared_voter_state,
        grandpa_shared_authority_set,
        grandpa_justification_stream,
        grandpa_finality_provider,
    } = grandpa;

    let EvmDeps {
        eth_filter_pool,
        eth_max_stored_filters,
        eth_backend,
        eth_max_past_logs,
        eth_fee_history_limit,
        eth_fee_history_cache,
        eth_overrides,
        eth_block_data_cache,
    } = evm;

    io.merge(System::new(Arc::clone(&client), Arc::clone(&pool), deny_unsafe).into_rpc())?;

    io.merge(TransactionPayment::new(Arc::clone(&client)).into_rpc())?;

    io.merge(
        Babe::new(
            Arc::clone(&client),
            babe_shared_epoch_changes,
            keystore,
            babe_config,
            select_chain,
            deny_unsafe,
        )
        .into_rpc(),
    )?;

    io.merge(
        Grandpa::new(
            Arc::clone(&subscription_task_executor),
            grandpa_shared_authority_set,
            grandpa_shared_voter_state,
            grandpa_justification_stream,
            grandpa_finality_provider,
        )
        .into_rpc(),
    )?;

    io.merge(
        AuthorExt::new(
            author_validator_key_extractor,
            Arc::clone(&client),
            Arc::clone(&pool),
            deny_unsafe,
        )
        .into_rpc(),
    )?;

    io.merge(
        Bioauth::new(
            robonode_client,
            bioauth_validator_key_extractor,
            bioauth_validator_signer_factory,
            Arc::clone(&client),
            Arc::clone(&pool),
            deny_unsafe,
        )
        .into_rpc(),
    )?;

    io.merge(
        Eth::new(
            Arc::clone(&client),
            Arc::clone(&pool),
            graph,
            Some(humanode_runtime::TransactionConverter),
            Arc::clone(&network),
            Vec::new(),
            Arc::clone(&eth_overrides),
            Arc::clone(&eth_backend),
            // Is authority.
            true,
            Arc::clone(&eth_block_data_cache),
            eth_fee_history_cache,
            eth_fee_history_limit,
        )
        .into_rpc(),
    )?;

    io.merge(Web3::new(Arc::clone(&client)).into_rpc())?;

    io.merge(
        EthPubSub::new(
            Arc::clone(&pool),
            Arc::clone(&client),
            Arc::clone(&network),
            Arc::clone(&subscription_task_executor),
            Arc::clone(&eth_overrides),
        )
        .into_rpc(),
    )?;

    io.merge(
        Net::new(
            Arc::clone(&client),
            Arc::clone(&network),
            // Whether to format the `peer_count` response as Hex (default) or not.
            true,
        )
        .into_rpc(),
    )?;

    if let Some(eth_filter_pool) = eth_filter_pool {
        io.merge(
            EthFilter::new(
                client,
                eth_backend,
                eth_filter_pool,
                eth_max_stored_filters,
                eth_max_past_logs,
                eth_block_data_cache,
            )
            .into_rpc(),
        )?;
    }

    Ok(io)
}
