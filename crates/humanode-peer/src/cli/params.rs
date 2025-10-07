//! Shared CLI parameters.

use crate::configuration::{EthTracingMode, FrontierBackendType};

/// Possible RPC URL scheme preference options.
#[derive(Debug, clap::ValueEnum, Clone)]
pub enum RpcUrlSchemePreference {
    /// Prefer HTTP (http or https).
    Http,
    /// Prefer WebSocket (ws or wss).
    Ws,
    /// No preference, use opinionated defaults.
    NoPreference,
}

/// Shared CLI parameters used to configure bioauth flow.
#[derive(Debug, clap::Parser, Clone)]
pub struct BioauthFlowParams {
    /// The URL to use for the web app.
    /// Used to print the QR Code to the console, so it doesn't matter much.
    #[arg(long, value_name = "WEBAPP_URL")]
    pub webapp_url: Option<String>,

    /// The URL to pass to the web app to connect to the node RPC.
    /// If not passed, a URL with WebSocket scheme and `localhost` path will be used.
    #[arg(long, value_name = "RPC_URL", conflicts_with_all = &["rpc_url_scheme_preference", "rpc_url_ngrok_detect", "rpc_url_unset"])]
    pub rpc_url: Option<String>,

    /// What RPC URL scheme to prefer.
    #[arg(
        long,
        value_enum,
        value_name = "RPC_URL_SCHEME_PREFERENCE",
        default_value = "no-preference",
        conflicts_with_all = &["rpc_url", "rpc_url_unset"]
    )]
    pub rpc_url_scheme_preference: RpcUrlSchemePreference,

    /// Detect RPC URL from ngrok.
    #[arg(long, conflicts_with_all = &["rpc_url", "rpc_url_unset"])]
    pub rpc_url_ngrok_detect: bool,

    /// Explicitly unset the RPC URL.
    #[arg(long, conflicts_with_all = &["rpc_url", "rpc_url_scheme_preference", "rpc_url_ngrok_detect"])]
    pub rpc_url_unset: bool,

    /// The tunnel name at ngrok to detect RPC URL from, if ngrok is used to detect the RPC URL.
    #[arg(long, value_name = "TUNNEL_NAME", default_value = "command_line")]
    pub rpc_url_ngrok_detect_from: String,

    /// The URL of robonode to authenticate with.
    #[arg(long, value_name = "ROBONODE_URL")]
    pub robonode_url: Option<String>,
}

/// Shared CLI parameters used to configure Ethereum RPC.
#[derive(Debug, clap::Parser, Clone)]
pub struct EthereumRpcParams {
    /// Maximum number of logs to keep from the latest block;
    /// it is not possible to query logs older than this amount from the latest block in the past.
    #[arg(long, default_value = "10000")]
    pub max_past_logs: u32,

    /// Maximum number of stored filters.
    #[arg(long, default_value = "500")]
    pub max_stored_filters: usize,

    /// Maximum fee history cache size.
    #[arg(long, default_value = "2048")]
    pub fee_history_limit: u64,

    /// A multiplier to allow larger gas limit in non-transactional execution.
    ///
    /// When using `eth_call/eth_estimateGas`, the maximum allowed gas limit will be
    /// `block.gas_limit` * `execute_gas_limit_multiplier`.
    #[arg(long, default_value = "10")]
    pub execute_gas_limit_multiplier: u64,

    /// Enable EVM tracing mode on a non-authority node.
    #[arg(long, value_delimiter = ',')]
    pub tracing_mode: Vec<EthTracingMode>,

    /// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
    #[arg(long, default_value = "10")]
    pub tracing_max_permits: u32,

    /// Size in bytes of data a raw tracing request is allowed to use.
    /// Bound the size of memory, stack and storage data.
    #[arg(long, default_value = "20000000")]
    pub tracing_debug_raw_max_memory_usage: usize,

    /// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
    /// A request asking for more or an unbounded one going over this limit will both return an
    /// error.
    #[arg(long, default_value = "500")]
    pub tracing_trace_max_count: u32,

    /// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
    /// discarded.
    #[arg(long, default_value = "300")]
    pub tracing_trace_cache_duration: u64,
}

/// Shared CLI parameters used to configure Frontier backend.
#[derive(Debug, Default, clap::Parser, Clone)]
pub struct FrontierBackendParams {
    /// Sets the frontier backend type (`KeyValue` or Sql).
    #[arg(long, value_enum, ignore_case = true, default_value_t = FrontierBackendType::default())]
    pub frontier_backend_type: FrontierBackendType,

    /// Sets the SQL backend's pool size.
    #[arg(long, default_value = "100")]
    pub frontier_sql_backend_pool_size: u32,

    /// Sets the SQL backend's query timeout in number of VM ops.
    #[arg(long, default_value = "10000000")]
    pub frontier_sql_backend_num_ops_timeout: u32,

    /// Sets the SQL backend's auxiliary thread limit.
    #[arg(long, default_value = "4")]
    pub frontier_sql_backend_thread_count: u32,

    /// Sets the SQL backend's query timeout in number of VM ops.
    /// Default value is 200MB.
    #[arg(long, default_value = "209715200")]
    pub frontier_sql_backend_cache_size: u64,
}

/// Shared CLI parameters used to configure time warp mode.
#[derive(Debug, clap::Parser, Clone)]
pub struct TimeWarpParams {
    /// The time in the future when the warp is going to be started.
    #[arg(long, requires = "time_warp_fork_timestamp")]
    pub time_warp_revive_timestamp: Option<u64>,

    /// The time of the last block that was finalized before the chain bricked.
    #[arg(long)]
    pub time_warp_fork_timestamp: Option<u64>,

    /// Warp factor that is going to be adopted.
    #[arg(long, requires = "time_warp_fork_timestamp")]
    pub time_warp_factor: Option<u64>,
}
