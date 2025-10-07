//! Humanode peer configuration.

use std::borrow::Cow;

use crate::{
    rpc_url::{RpcUrl, RpcUrlResolver},
    time_warp::TimeWarp,
};

/// Peer configuration, including both standard substrate configuration and our custom extensions.
pub struct Configuration {
    /// Standard substrate configuration.
    pub substrate: sc_service::Configuration,

    /// Bioauth flow configuration.
    /// If not defined, bioauth flows are unavailable.
    /// A lot of operations do not involve bioauth flows, so the configuration is not
    /// always required.
    pub bioauth_flow: Option<BioauthFlow>,

    /// Ethereum RPC configuration.
    pub ethereum_rpc: Option<EthereumRpc>,

    /// Frontier backend configuration.
    pub frontier_backend: FrontierBackend,

    /// Time warp mode configuration.
    /// If not defined, time warp mode isn't enabled.
    pub time_warp: Option<TimeWarp>,
}

/// Bioauth flow configuration parameters.
pub struct BioauthFlow {
    /// The RPC URL Resolver.
    /// Used for detecting the RPC URL in non-trivial cases.
    pub rpc_url_resolver: RpcUrlResolver,

    /// The URL to use for the web app.
    /// Used to print the QR Code to the console, so it can be optional, and if it's not defined
    /// we can assume user will take care of figuring out how to authenticate on its own.
    pub webapp_url: Option<String>,

    /// The URL to pass to the webapp to connect to the node RPC.
    /// If it's not defined we can assume user will take care of figuring out how
    /// to authenticate on its own.
    pub rpc_url: RpcUrl,

    /// The URL of robonode to authenticate with.
    pub robonode_url: String,
}

impl BioauthFlow {
    /// Obtain QR Code URL params.
    pub async fn qrcode_params(&self) -> Result<(&str, Cow<'_, str>), Cow<'static, str>> {
        let webapp_url = self.webapp_url.as_deref().ok_or("webapp URL is not set")?;
        let rpc_url = self.rpc_url_resolver.resolve(&self.rpc_url).await?;
        Ok((webapp_url, rpc_url))
    }
}

/// Ethereum RPC configuration parameters.
pub struct EthereumRpc {
    /// Maximum number of blocks to keep the log information available
    /// for querying via the RPC (from the latest block).
    pub max_past_logs: u32,

    /// Maximum number of stored filters.
    pub max_stored_filters: usize,

    /// Maximum fee history cache size.
    pub fee_history_limit: u64,

    /// A multiplier to allow larger gas limit in non-transactional execution.
    ///
    /// When using `eth_call/eth_estimateGas`, the maximum allowed gas limit will be
    /// `block.gas_limit` * `execute_gas_limit_multiplier`.
    pub execute_gas_limit_multiplier: u64,

    /// Enable EVM tracing mode on a non-authority node.
    pub tracing_mode: Vec<EthTracingMode>,

    /// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
    pub tracing_max_permits: u32,

    /// Size in bytes of data a raw tracing request is allowed to use.
    /// Bound the size of memory, stack and storage data.
    pub tracing_debug_raw_max_memory_usage: usize,

    /// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
    /// A request asking for more or an unbounded one going over this limit will both return an
    /// error.
    pub tracing_trace_max_count: u32,

    /// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
    /// discarded.
    pub tracing_trace_cache_duration: u64,
}

/// Frontier backend configuration parameters.
pub struct FrontierBackend {
    /// Sets the frontier backend type (`KeyValue` or Sql).
    pub frontier_backend_type: FrontierBackendType,

    /// Sets the SQL backend's pool size.
    pub frontier_sql_backend_pool_size: u32,

    /// Sets the SQL backend's query timeout in number of VM ops.
    pub frontier_sql_backend_num_ops_timeout: u32,

    /// Sets the SQL backend's auxiliary thread limit.
    pub frontier_sql_backend_thread_count: u32,

    /// Sets the SQL backend's query timeout in number of VM ops.
    /// Default value is 200MB.
    pub frontier_sql_backend_cache_size: u64,
}

/// Avalailable frontier backend types.
#[derive(Default, Debug, Copy, Clone, clap::ValueEnum)]
pub enum FrontierBackendType {
    /// Either `RocksDb` or `ParityDb` as per inherited from the global backend settings.
    #[default]
    KeyValue,
    /// Sql database with custom log indexing.
    Sql,
}

/// Possible EVM tracing modes.
#[derive(Debug, clap::ValueEnum, Clone, PartialEq)]
pub enum EthTracingMode {
    /// Debug mode.
    Debug,
    /// Trace mode.
    Trace,
}
