//! Initializing, bootstrapping and launching the node from a provided configuration.

#![allow(clippy::type_complexity)]
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use fc_consensus::FrontierBlockImport;
use fc_mapping_sync::{MappingSyncWorker, SyncStrategy};
use fc_rpc::EthTask;
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use futures::StreamExt;
use humanode_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::{BlockBackend, BlockchainEvents};
use sc_consensus_babe::SlotProportion;
pub use sc_executor::NativeElseWasmExecutor;
use sc_finality_grandpa::SharedVoterState;
use sc_service::{Error as ServiceError, KeystoreContainer, PartialComponents, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use tracing::*;

use crate::configuration::Configuration;

pub mod frontier;
pub mod inherents;

/// Declare an instance of the native executor named `ExecutorDispatch`. Include the wasm binary as
/// the equivalent wasm code.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        humanode_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        humanode_runtime::native_version()
    }
}

/// Keystore bioauth identifier used at the keystore.
pub type KeystoreBioauthId = keystore_bioauth_account_id::KeystoreBioauthAccountId;
/// Executor type.
type Executor = NativeElseWasmExecutor<ExecutorDispatch>;
/// Full node client type.
pub(crate) type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
/// Full node backend type.
type FullBackend = sc_service::TFullBackend<Block>;
/// Full node select chain type.
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
/// Full type for `GrandpaBlockImport`.
type FullGrandpa =
    sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;
/// Full type for `BabeBlockImport`.
type FullBabe = sc_consensus_babe::BabeBlockImport<Block, FullClient, FullGrandpa>;
/// Full type for `FrontierBlockImport`.
type FullFrontier = FrontierBlockImport<Block, FullBabe, FullClient>;
/// Whatever we currently use as the block import.
type EffectiveFullBlockImport = FullFrontier;
/// Frontier backend type.
type FrontierBackend = fc_db::Backend<Block>;

/// Construct a bare keystore from the configuration.
pub fn keystore_container(
    config: &Configuration,
) -> Result<(KeystoreContainer, TaskManager), ServiceError> {
    let executor = Executor::new(
        config.substrate.wasm_method,
        config.substrate.default_heap_pages,
        config.substrate.max_runtime_instances,
        config.substrate.runtime_cache_size,
    );

    let (_client, _backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(&config.substrate, None, executor)?;
    Ok((keystore_container, task_manager))
}

/// Extract substrate partial components.
pub fn new_partial(
    config: &Configuration,
) -> Result<
    PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block, FullClient>,
        sc_transaction_pool::FullPool<Block, FullClient>,
        (
            sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
            sc_consensus_babe::BabeLink<Block>,
            EffectiveFullBlockImport,
            inherents::Creator<FullClient>,
            Arc<FrontierBackend>,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
> {
    let Configuration {
        substrate: config,
        evm: evm_config,
        time_warp: time_warp_config,
        ..
    } = config;

    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = Executor::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        Arc::clone(&client),
    );

    let select_chain = sc_consensus::LongestChain::new(Arc::clone(&backend));

    let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
        Arc::clone(&client),
        &(Arc::clone(&client) as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::configuration(&*client)?,
        grandpa_block_import.clone(),
        Arc::clone(&client),
    )?;

    let frontier_backend = Arc::new(FrontierBackend::open(
        Arc::clone(&client),
        &config.database,
        &frontier::db_config_dir(config),
    )?);
    let frontier_block_import = FrontierBlockImport::new(
        babe_block_import,
        Arc::clone(&client),
        Arc::clone(&frontier_backend),
    );

    let raw_slot_duration = babe_link.config().slot_duration();
    let eth_target_gas_price = evm_config.target_gas_price;
    let inherent_data_providers_creator = inherents::Creator {
        raw_slot_duration,
        eth_target_gas_price,
        client: Arc::clone(&client),
        time_warp: time_warp_config.clone(),
    };

    let import_queue = sc_consensus_babe::import_queue(
        babe_link.clone(),
        frontier_block_import.clone(),
        Some(Box::new(grandpa_block_import)),
        Arc::clone(&client),
        select_chain.clone(),
        inherents::ForImport(inherent_data_providers_creator.clone()),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;
    Ok(PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (
            grandpa_link,
            babe_link,
            frontier_block_import,
            inherent_data_providers_creator,
            frontier_backend,
            telemetry,
        ),
    })
}

/// Create a "full" node (full is in terms of substrate).
/// We don't support other node types yet either way, so this is the only way to create a node.
pub async fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other:
            (
                grandpa_link,
                babe_link,
                block_import,
                inherent_data_providers_creator,
                frontier_backend,
                mut telemetry,
            ),
    } = new_partial(&config)?;
    let Configuration {
        substrate: mut config,
        bioauth_flow: bioauth_flow_config,
        evm: _evm_config,
        ethereum_rpc: ethereum_rpc_config,
        ..
    } = config;

    let grandpa_protocol_name = sc_finality_grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );

    config
        .network
        .extra_sets
        .push(sc_finality_grandpa::grandpa_peers_set_config(
            grandpa_protocol_name.clone(),
        ));

    let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
        Arc::clone(&backend),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));

    let bioauth_flow_config = bioauth_flow_config
        .ok_or_else(|| ServiceError::Other("bioauth flow config is not set".into()))?;

    let ethereum_rpc_config = ethereum_rpc_config
        .ok_or_else(|| ServiceError::Other("Ethereum RPC config is not set".into()))?;

    let role = config.role.clone();
    let name = config.network.node_name.clone();
    let keystore = Some(keystore_container.sync_keystore());
    let enable_grandpa = !config.disable_grandpa;
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let prometheus_registry = config.prometheus_registry().cloned();
    let eth_filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
    let eth_fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));
    let eth_fee_history_limit = ethereum_rpc_config.fee_history_limit;
    let eth_overrides = fc_storage::overrides_handle(Arc::clone(&client));

    let proposer_factory = sc_basic_authorship::ProposerFactory::new(
        task_manager.spawn_handle(),
        Arc::clone(&client),
        Arc::clone(&transaction_pool),
        config.prometheus_registry(),
        telemetry.as_ref().map(|x| x.handle()),
    );

    let account_validator_key_extractor =
        Arc::new(bioauth_keys::KeyExtractor::<KeystoreBioauthId, _>::new(
            keystore_container.sync_keystore(),
            bioauth_keys::OneOfOneSelector,
        ));

    let (network, system_rpc_tx, tx_handler_controller, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: Arc::clone(&client),
            transaction_pool: Arc::clone(&transaction_pool),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync: Some(warp_sync),
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            Arc::clone(&client),
            Arc::clone(&network) as _,
        );
    }

    let robonode_client = Arc::new(robonode_client::Client {
        base_url: bioauth_flow_config.robonode_url.clone(),
        reqwest: reqwest::Client::new(),
    });

    let rpc_extensions_builder = {
        let client = Arc::clone(&client);
        let pool = Arc::clone(&transaction_pool);
        let robonode_client = Arc::clone(&robonode_client);
        let is_authority = role.is_authority();
        let bioauth_validator_key_extractor = Arc::clone(&account_validator_key_extractor);
        let bioauth_validator_signer_factory = {
            let keystore = keystore_container.keystore();
            Arc::new(move |key| {
                crate::validator_key::AppCryptoSigner::new(
                    Arc::clone(&keystore),
                    crate::validator_key::AppCryptoPublic(key),
                )
            })
        };
        let network = Arc::clone(&network);

        let grandpa_justification_stream = grandpa_link.justification_stream();
        let grandpa_shared_authority_set = grandpa_link.shared_authority_set().clone();
        let grandpa_shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();
        let grandpa_finality_proof_provider =
            sc_finality_grandpa::FinalityProofProvider::new_for_service(
                Arc::clone(&backend),
                Some(grandpa_shared_authority_set.clone()),
            );

        let babe_config = babe_link.config().clone();
        let babe_shared_epoch_changes = babe_link.epoch_changes().clone();

        let keystore = keystore_container.sync_keystore();
        let chain_spec = config.chain_spec.cloned_box();
        let select_chain = select_chain.clone();

        let eth_filter_pool = eth_filter_pool.clone();
        let frontier_backend = Arc::clone(&frontier_backend);
        let eth_overrides = Arc::clone(&eth_overrides);
        let eth_block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
            task_manager.spawn_handle(),
            Arc::clone(&eth_overrides),
            50,
            50,
            config.prometheus_registry().cloned(),
        ));
        let eth_fee_history_cache = Arc::clone(&eth_fee_history_cache);

        Box::new(move |deny_unsafe, subscription_task_executor| {
            Ok(humanode_rpc::create(humanode_rpc::Deps {
                client: Arc::clone(&client),
                pool: Arc::clone(&pool),
                deny_unsafe,
                graph: Arc::clone(pool.pool()),
                network: Arc::clone(&network),
                chain_spec: chain_spec.cloned_box(),
                author_ext: humanode_rpc::AuthorExtDeps {
                    author_validator_key_extractor: Arc::clone(&bioauth_validator_key_extractor),
                },
                is_authority,
                bioauth: humanode_rpc::BioauthDeps {
                    robonode_client: Arc::clone(&robonode_client),
                    bioauth_validator_signer_factory: Arc::clone(&bioauth_validator_signer_factory),
                    bioauth_validator_key_extractor: Arc::clone(&bioauth_validator_key_extractor),
                },
                babe: humanode_rpc::BabeDeps {
                    babe_config: babe_config.clone(),
                    babe_shared_epoch_changes: babe_shared_epoch_changes.clone(),
                    keystore: Arc::clone(&keystore),
                },
                grandpa: humanode_rpc::GrandpaDeps {
                    grandpa_shared_voter_state: grandpa_shared_voter_state.clone(),
                    grandpa_shared_authority_set: grandpa_shared_authority_set.clone(),
                    grandpa_justification_stream: grandpa_justification_stream.clone(),
                    grandpa_finality_provider: Arc::clone(&grandpa_finality_proof_provider),
                },
                select_chain: select_chain.clone(),
                evm: humanode_rpc::EvmDeps {
                    eth_filter_pool: eth_filter_pool.clone(),
                    eth_max_stored_filters: ethereum_rpc_config.max_stored_filters,
                    eth_backend: Arc::clone(&frontier_backend),
                    eth_max_past_logs: ethereum_rpc_config.max_past_logs,
                    eth_fee_history_limit,
                    eth_fee_history_cache: Arc::clone(&eth_fee_history_cache),
                    eth_overrides: Arc::clone(&eth_overrides),
                    eth_block_data_cache: Arc::clone(&eth_block_data_cache),
                    eth_execute_gas_limit_multiplier: ethereum_rpc_config
                        .execute_gas_limit_multiplier,
                },
                subscription_task_executor,
            })?)
        })
    };

    {
        let keystore = keystore_container.sync_keystore();
        init_dev_bioauth_keystore_keys(keystore.as_ref(), config.dev_key_seed.as_deref())
            .map_err(|err| sc_service::Error::Other(err.to_string()))?;
    }

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: Arc::clone(&network) as _,
        client: Arc::clone(&client),
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool: Arc::clone(&transaction_pool),
        rpc_builder: rpc_extensions_builder,
        backend: Arc::clone(&backend),
        system_rpc_tx,
        tx_handler_controller,
        config,
        telemetry: telemetry.as_mut(),
    })?;

    let babe_config = sc_consensus_babe::BabeParams {
        keystore: keystore_container.sync_keystore(),
        client: Arc::clone(&client),
        select_chain,
        env: proposer_factory,
        block_import,
        sync_oracle: Arc::clone(&network),
        justification_sync_link: Arc::clone(&network),
        create_inherent_data_providers: inherents::ForProduction(inherent_data_providers_creator),
        force_authoring,
        backoff_authoring_blocks,
        babe_link,
        block_proposal_slot_portion: SlotProportion::new(0.5),
        max_block_proposal_slot_portion: None,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
    };

    let babe = sc_consensus_babe::start_babe(babe_config)?;
    task_manager.spawn_essential_handle().spawn_blocking(
        "babe-proposer",
        Some("block-authoring"),
        babe,
    );

    let grandpa_config = sc_finality_grandpa::Config {
        // See substrate#1578: make this available through chainspec.
        // Ref: https://github.com/paritytech/substrate/issues/1578
        gossip_duration: Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
        protocol_name: grandpa_protocol_name,
    };

    if enable_grandpa {
        let grandpa_config = sc_finality_grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            network,
            voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state: SharedVoterState::empty(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        };

        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            Some("block-finalization"),
            sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    task_manager.spawn_essential_handle().spawn_blocking(
        "frontier-mapping-sync-worker",
        Some("evm"),
        MappingSyncWorker::new(
            client.import_notification_stream(),
            Duration::from_millis(humanode_runtime::SLOT_DURATION),
            Arc::clone(&client),
            backend,
            Arc::clone(&eth_overrides),
            frontier_backend,
            // retry_times: usize,
            3,
            // sync_from: <Block::Header as HeaderT>::Number,
            0,
            SyncStrategy::Normal,
        )
        .for_each(|()| futures::future::ready(())),
    );

    // Spawn Frontier FeeHistory cache maintenance task.
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("evm"),
        EthTask::fee_history_task(
            Arc::clone(&client),
            Arc::clone(&eth_overrides),
            eth_fee_history_cache,
            eth_fee_history_limit,
        ),
    );

    // Spawn Frontier EthFilterApi maintenance task.
    if let Some(eth_filter_pool) = eth_filter_pool {
        /// Each filter is allowed to stay in the pool for 100 blocks.
        const FILTER_RETAIN_THRESHOLD: u64 = 100;
        task_manager.spawn_essential_handle().spawn(
            "frontier-filter-pool",
            Some("evm"),
            EthTask::filter_pool_task(
                Arc::clone(&client),
                eth_filter_pool,
                FILTER_RETAIN_THRESHOLD,
            ),
        );
    }

    network_starter.start_network();

    let webapp_qrcode = bioauth_flow_config
        .qrcode_params()
        .await
        .and_then(|(webapp_url, rpc_url)| crate::qrcode::WebApp::new(webapp_url, rpc_url));

    match &webapp_qrcode {
        Ok(ref qrcode) => qrcode.print(),
        Err(ref err) => {
            error!("Bioauth flow - unable to display QR Code: {}", err);
        }
    };

    Ok(task_manager)
}

/// Initialize the keystore with the [`KeystoreBioauthId`] key from the dev seed.
///
/// This is analogous to `sp_session::generate_initial_session_keys` which
/// executes as part of [`sc_service::spawn_tasks`] - but for the [`KeystoreBioauthId`], which
/// is not a part of the session keys set, so it wont be populated that way.
///
/// We need [`KeystoreBioauthId`] for the block production though, so we initialize it manually.
fn init_dev_bioauth_keystore_keys<Keystore: sp_keystore::SyncCryptoStore + ?Sized>(
    keystore: &Keystore,
    seed: Option<&str>,
) -> Result<(), sp_keystore::Error> {
    if let Some(seed) = seed {
        use sp_application_crypto::AppKey;
        let _public = sp_keystore::SyncCryptoStore::sr25519_generate_new(
            keystore,
            KeystoreBioauthId::ID,
            Some(seed),
        )?;
    }
    Ok(())
}
