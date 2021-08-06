//! Provides the [`Configuration`].

use futures::FutureExt;
use sc_executor::WasmExecutionMethod;
use sc_network::{config::*, multiaddr::Protocol};
use sc_service::{
    config::*, Configuration, DatabaseConfig, KeepBlocks, PruningMode, Role, RpcMethods,
    TracingReceiver, TransactionPoolOptions, TransactionStorageMode,
};
use sc_transaction_pool::txpool::base_pool::Limit;

/// Create and return a standalone configuration.
/// This is mostly hardcoded for now, for simplicity.
/// We later on plan to switch to a cli interface to enable more flexible configuration.
pub fn make() -> Configuration {
    // Set the settings.
    let name = std::env::var("NODE_NAME").unwrap_or_else(|_| "default".into());
    let version = "0".to_owned();

    let chain_spec = crate::chain_spec::local_testnet_config().unwrap();

    // Use current tokio runtime.
    let tokio_runtime_handle = tokio::runtime::Handle::current();
    Configuration {
        impl_name: "humanode".to_owned(),
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
            listen_addresses: vec![Multiaddr::empty()
                .with(Protocol::Ip4([0, 0, 0, 0].into()))
                .with(Protocol::Tcp(
                    std::env::var("LISTEN_ADDRESS_PORT")
                        .unwrap_or_else(|_| "30333".into())
                        .parse::<u16>()
                        .unwrap(),
                ))],
            public_addresses: vec![],
            // TODO: `boot_nodes` should probably be configurable by the user, rather than be hardcoded
            // or empty.
            boot_nodes: load_boot_nodes(),
            // TODO: take a deeper look into this and discuss.
            node_key: NodeKeyConfig::default(),
            request_response_protocols: vec![],
            default_peers_set: SetConfig {
                in_peers: 10_000,
                out_peers: 10_000,
                reserved_nodes: vec![],
                non_reserved_mode: NonReservedPeerMode::Accept,
            },
            extra_sets: vec![],
            client_version: version,
            node_name: name,
            transport: TransportConfig::Normal {
                allow_private_ipv4: true,
                enable_mdns: false,
                wasm_external_transport: None,
            },
            max_parallel_downloads: 64,
            enable_dht_random_walk: true,
            allow_non_globals_in_dht: false,
            kademlia_disjoint_query_paths: true,
            ipfs_server: true,
            yamux_window_size: None,
        },
        keystore: KeystoreConfig::Path {
            password: None,
            path: format!(
                "{}/humanode/keystore",
                std::env::var("HUMANODE_PATH").unwrap_or_else(|_| "/tmp/humanode_default".into())
            )
            .into(),
        },
        keystore_remote: None,
        database: DatabaseConfig::RocksDb {
            path: format!(
                "{}/humanode/database",
                std::env::var("HUMANODE_PATH").unwrap_or_else(|_| "/tmp/humanode_default".into())
            )
            .into(),
            cache_size: 100,
        },
        state_cache_size: 1000,
        state_cache_child_ratio: None,
        state_pruning: PruningMode::ArchiveAll,
        keep_blocks: KeepBlocks::All,
        transaction_storage: TransactionStorageMode::BlockBody,
        chain_spec: Box::new(chain_spec),
        wasm_method: WasmExecutionMethod::Interpreted,
        wasm_runtime_overrides: None,
        execution_strategies: Default::default(),
        rpc_http: Some(
            std::env::var("RPC_HTTP")
                .unwrap_or_else(|_| "127.0.0.1:9933".into())
                .parse()
                .unwrap(),
        ),
        rpc_ws: Some(
            std::env::var("RPC_WS")
                .unwrap_or_else(|_| "127.0.0.1:9944".into())
                .parse()
                .unwrap(),
        ),
        rpc_ipc: None,
        rpc_ws_max_connections: None,
        rpc_cors: None,
        rpc_methods: RpcMethods::Safe,
        prometheus_config: Some(PrometheusConfig {
            port: std::env::var("PROMETHEUS_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:5959".into())
                .parse()
                .unwrap(),
            registry: Default::default(),
        }),
        telemetry_endpoints: None,
        telemetry_external_transport: None,
        default_heap_pages: None,
        offchain_worker: OffchainWorkerConfig {
            enabled: false,
            indexing_enabled: false,
        },
        force_authoring: false,
        disable_grandpa: true,
        dev_key_seed: Some(std::env::var("DEV_KEY_SEED").unwrap_or_else(|_| "//Default".into())),
        tracing_targets: None,
        disable_log_reloading: true,
        tracing_receiver: TracingReceiver::Log,
        max_runtime_instances: 8,
        announce_block: true,
        base_path: None,
        informant_output_format: Default::default(),
    }
}

/// A helper function to extract boot nodes.
fn load_boot_nodes() -> Vec<MultiaddrWithPeerId> {
    std::env::var("BOOT_NODES")
        .unwrap_or_else(|_| "".into())
        .split_whitespace()
        .into_iter()
        .map(|addr| addr.parse().unwrap())
        .collect::<Vec<MultiaddrWithPeerId>>()
}
