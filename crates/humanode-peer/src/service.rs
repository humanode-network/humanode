use std::sync::Arc;

use sc_service::{Configuration, Error as ServiceError, TaskManager};
use sp_runtime::traits::Block as BlockT;

/// From runtime.
type Block = ();
type RuntimeApi = ();
type Executor = ();

type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
// type FullSelectChain = DummyConsensus<Block>;

pub fn new_full(mut config: Configuration) -> Result<TaskManager, ServiceError> {
    let inherent_data_providers = sp_inherents::InherentDataProviders::new();

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
    let client = Arc::new(client);

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_handle(),
        Arc::clone(&client),
    );

    let (network, network_status_sinks, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: None,
            block_announce_validator_builder: None,
        })?;

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    let rpc_extensions_builder = Box::new(sc_service::NoopRpcExtensionBuilder(()))
        as Box<dyn sc_service::RpcExtensionBuilder<Output = ()>>;

    let (_rpc_handlers, telemetry_connection_notifier) =
        sc_service::spawn_tasks(sc_service::SpawnTasksParams {
            network: network.clone(),
            client: client.clone(),
            keystore: keystore_container.sync_keystore(),
            task_manager: &mut task_manager,
            transaction_pool: transaction_pool.clone(),
            rpc_extensions_builder,
            on_demand: None,
            remote_blockchain: None,
            backend,
            network_status_sinks,
            system_rpc_tx,
            config,
        })?;

    network_starter.start_network();
    Ok(task_manager)
}
