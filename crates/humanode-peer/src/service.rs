//! Initializing, bootstrapping and launching the node from a provided configuration.

use std::sync::Arc;

use humanode_runtime::{self, opaque::Block, RuntimeApi};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_service::{Configuration, Error as ServiceError, TaskManager};
use sp_consensus::import_queue::BasicQueue;

use crate::dummy::DummyVerifier;

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

    let import_queue = BasicQueue::new(
        DummyVerifier,
        Box::new(Arc::clone(&client)),
        None,
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
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

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network,
        client,
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool,
        rpc_extensions_builder: Box::new(|_, _| jsonrpc_core::IoHandler::default()),
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
