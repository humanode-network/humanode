//! Initializing, bootstrapping and launching the node from a provided configuration.

use std::sync::Arc;

use humanode_runtime::{self, opaque::Block, RuntimeApi};
use sc_consensus_manual_seal::InstantSealParams;
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_service::{Configuration, Error as ServiceError, TaskManager};

// Native executor for the runtime based on the runtime API that is available
// at the current compile time.
native_executor_instance!(
    pub Executor,
    humanode_runtime::api::dispatch,
    humanode_runtime::native_version,
);

/// Create a "full" node (full is in terms of substrate).
/// We don't support other node types yet either way, so this is the only way to create a node.
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    let (client, backend, keystore_container, mut task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config, None)?;
    let client = Arc::new(client);

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_handle(),
        Arc::clone(&client),
    );

    let import_queue = sc_consensus_manual_seal::import_queue(
        Box::new(Arc::clone(&client)),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    );

    let proposer_factory = sc_basic_authorship::ProposerFactory::new(
        task_manager.spawn_handle(),
        Arc::clone(&client),
        Arc::clone(&transaction_pool),
        config.prometheus_registry(),
        None,
    );

    let authorship_future = sc_consensus_manual_seal::run_instant_seal(InstantSealParams {
        block_import: Arc::clone(&client),
        env: proposer_factory,
        client: Arc::clone(&client),
        pool: Arc::clone(transaction_pool.pool()),
        select_chain: sc_consensus::LongestChain::new(Arc::clone(&backend)),
        consensus_data_provider: None,
        create_inherent_data_providers: move |_, ()| async move { Ok(()) },
    });

    task_manager
        .spawn_essential_handle()
        .spawn_blocking("instant-seal", authorship_future);

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: Arc::clone(&client),
            transaction_pool: Arc::clone(&transaction_pool),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: None,
            block_announce_validator_builder: None,
        })?;

    let robonode_url =
        std::env::var("ROBONODE_URL").unwrap_or_else(|_| "http://127.0.0.1:3033".into());
    let robonode_client = Arc::new(robonode_client::Client {
        base_url: robonode_url,
        reqwest: reqwest::Client::new(),
    });

    let (bioauth_flow_rpc_slot, _bioauth_flow_provider_slot) =
        bioauth_flow::rpc::new_liveness_data_tx_slot();

    let rpc_extensions_builder = {
        let client = Arc::clone(&client);
        let pool = Arc::clone(&transaction_pool);
        let robonode_client = Arc::clone(&robonode_client);
        let bioauth_flow_rpc_slot = Arc::new(bioauth_flow_rpc_slot);
        Box::new(move |deny_unsafe, _| {
            humanode_rpc::create(humanode_rpc::Deps {
                client: Arc::clone(&client),
                pool: Arc::clone(&pool),
                deny_unsafe,
                robonode_client: Arc::clone(&robonode_client),
                bioauth_flow_slot: Arc::clone(&bioauth_flow_rpc_slot),
            })
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network,
        client,
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool,
        rpc_extensions_builder,
        on_demand: None,
        remote_blockchain: None,
        backend,
        system_rpc_tx,
        config,
        telemetry: None,
    })?;

    network_starter.start_network();
    Ok(task_manager)
}
