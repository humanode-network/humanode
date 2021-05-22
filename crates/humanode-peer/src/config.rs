use futures::FutureExt;
use sc_service::{Configuration, DatabaseConfig, KeepBlocks, PruningMode, Role, TransactionPoolOptions, TransactionStorageMode, config::{
        KeystoreConfig, Multiaddr, NetworkConfiguration, NodeKeyConfig, SetConfig, TransportConfig,
    }};
use sc_transaction_pool::txpool::base_pool::Limit;

pub fn make() -> Configuration {
    // Set the settings.
    let name = "humanode".to_owned();
    let version = "0".to_owned();

    // Use current tokio runtime.
    let tokio_runtime_handle = tokio::runtime::Handle::current();
    Configuration {
        impl_name: name.clone(),
        impl_version: version.clone(),
        role: Role::Full,
        task_executor: (move |future, _task_type| {
            // Spawn the task onto the tokio runtime.
            tokio_runtime_handle.spawn(future).map(|result|
                    // This result can only be an error if the task at tokio panicked.
                    // Here we propagate the panics to the task that will be `.await`ing
                    // (or manually polling) the future.
                    // But really this is a bad interface, and [`sc_service::TaskExecutor`]
                    // should allow for a [`Result`] to be returned.
                    result.expect("panic propagation from the tokio runtime"))
        })
        .into(),
        // TODO: adjust these values.
        transaction_pool: TransactionPoolOptions {
            ready: Limit {
                count: 10_000,
                total_bytes: 1024 * 1024 * 1_000, // 1 000 MiB
            },
            future: Limit {
                count: 0,
                total_bytes: 0,
            },
            reject_future_transactions: true,
        },
        // TODO: tweak these parameters.
        network: NetworkConfiguration {
            net_config_path: None,
            listen_addresses: vec![Multiaddr::empty()],
            public_addresses: vec![Multiaddr::empty()],
            // TODO: `boot_nodes` should probably be configurable by the user, rather than be hardcoded
            // or empty.
            boot_nodes: vec![],
            // TODO: take a deeper look into this and discuss.
            node_key: NodeKeyConfig::default(),
            request_response_protocols: vec![],
            default_peers_set: SetConfig {},
            extra_sets: vec![],
            client_version: version.clone(),
            node_name: name.clone(),
            transport: TransportConfig {},
            max_parallel_downloads: 64,
            enable_dht_random_walk: true,
            allow_non_globals_in_dht: false,
            kademlia_disjoint_query_paths: true,
            ipfs_server: true,
            yamux_window_size: None,
        },
        keystore: KeystoreConfig::Path {
            password: None,
            path: "/tmp/numanode/keystore".into(),
        },
        keystore_remote: None,
        database: DatabaseConfig::RocksDb {
            path: "/tmp/numanode/database".into(),
            cache_size: 100,
        },
        state_cache_size: 1000,
        state_cache_child_ratio: None,
        state_pruning: PruningMode::ArchiveAll,
        keep_blocks: KeepBlocks::All,
        transaction_storage: TransactionStorageMode::BlockBody,
        chain_spec: (),
        wasm_method: (),
        wasm_runtime_overrides: (),
        execution_strategies: (),
        rpc_http: (),
        rpc_ws: (),
        rpc_ipc: (),
        rpc_ws_max_connections: (),
        rpc_cors: (),
        rpc_methods: (),
        prometheus_config: (),
        telemetry_endpoints: (),
        telemetry_external_transport: (),
        telemetry_handle: (),
        telemetry_span: (),
        default_heap_pages: (),
        offchain_worker: (),
        force_authoring: (),
        disable_grandpa: (),
        dev_key_seed: (),
        tracing_targets: (),
        disable_log_reloading: (),
        tracing_receiver: (),
        max_runtime_instances: (),
        announce_block: (),
        base_path: (),
        informant_output_format: (),
    }
}
