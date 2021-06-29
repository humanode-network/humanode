//! Initializing, bootstrapping and launching the node from a provided configuration.

use std::sync::Arc;

use humanode_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::ExecutorProvider;
use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_service::{Configuration, Error as ServiceError, TaskManager};
use sp_consensus::SlotData;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

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

    let select_chain = sc_consensus::LongestChain::new(Arc::clone(&backend));

    let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
    let raw_slot_duration = slot_duration.slot_duration();
    let can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;

    let import_queue =
        sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(ImportQueueParams {
            block_import: Arc::clone(&client),
            justification_import: None,
            client: Arc::clone(&client),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        raw_slot_duration,
                    );

                Ok((timestamp, slot))
            },
            spawner: &task_manager.spawn_essential_handle(),
            can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
                client.executor().clone(),
            ),
            registry: config.prometheus_registry(),
            check_for_equivocation: Default::default(),
            telemetry: None,
        })?;

    let proposer_factory = sc_basic_authorship::ProposerFactory::new(
        task_manager.spawn_handle(),
        Arc::clone(&client),
        Arc::clone(&transaction_pool),
        config.prometheus_registry(),
        None,
    );

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
                bioauth_runtime_handle: tokio::runtime::Handle::current(),
            })
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: Arc::clone(&network),
        client: Arc::clone(&client),
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

    let aura =
        sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _>(StartAuraParams {
            slot_duration,
            client: Arc::clone(&client),
            select_chain,
            block_import: client,
            proposer_factory,
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        raw_slot_duration,
                    );

                Ok((timestamp, slot))
            },
            force_authoring,
            backoff_authoring_blocks,
            keystore: keystore_container.sync_keystore(),
            can_author_with,
            sync_oracle: network,
            block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
            telemetry: None,
        })?;

    // The AURA authoring task is considered essential, i.e. if it
    // fails we take down the service with it.
    task_manager
        .spawn_essential_handle()
        .spawn_blocking("aura", aura);

    network_starter.start_network();

    Ok(task_manager)
}
