//! Shared CLI parameters.

/// Possible RPC URL scheme preference options.
#[derive(Debug, clap::ArgEnum, Clone)]
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
    #[clap(long, value_name = "WEBAPP_URL")]
    pub webapp_url: Option<String>,

    /// The URL to pass to the web app to connect to the node RPC.
    /// If not passed, a URL with `localhost` and the HTTP RPC port will be used.
    #[clap(long, value_name = "RPC_URL", conflicts_with_all = &["rpc-url-scheme-preference", "rpc-url-ngrok-detect", "rpc-url-unset"])]
    pub rpc_url: Option<String>,

    /// What RPC URL scheme to prefer.
    #[clap(
        long,
        arg_enum,
        value_name = "RPC_URL_SCHEME_PREFERENCE",
        default_value = "no-preference",
        conflicts_with_all = &["rpc-url", "rpc-url-unset"]
    )]
    pub rpc_url_scheme_preference: RpcUrlSchemePreference,

    /// Detect RPC URL from ngrok.
    #[clap(long, conflicts_with_all = &["rpc-url", "rpc-url-unset"])]
    pub rpc_url_ngrok_detect: bool,

    /// Explicitly unset the RPC URL.
    #[clap(long, conflicts_with_all = &["rpc-url", "rpc-url-scheme-preference", "rpc-url-ngrok-detect"])]
    pub rpc_url_unset: bool,

    /// The tunnel name at ngrok to detect RPC URL from, if ngrok is used to detect the RPC URL.
    #[clap(long, value_name = "TUNNEL_NAME", default_value = "command_line")]
    pub rpc_url_ngrok_detect_from: String,

    /// The URL of robonode to authenticate with.
    #[clap(long, value_name = "ROBONODE_URL")]
    pub robonode_url: Option<String>,
}

/// Shared CLI parameters used to configure EVM.
#[derive(Debug, clap::Parser, Clone)]
pub struct EvmParams {
    /// The dynamic-fee pallet target gas price set by block author.
    #[clap(long, default_value = "1")]
    pub target_gas_price: u64,
}

/// Shared CLI parameters used to configure Ethereum RPC.
#[derive(Debug, clap::Parser, Clone)]
pub struct EthereumRpcParams {
    /// Maximum number of logs to keep from the latest block;
    /// it is not possible to query logs older than this amount from the latest block in the past.
    #[clap(long, default_value = "10000")]
    pub max_past_logs: u32,

    /// Maximum number of stored filters.
    #[clap(long, default_value = "500")]
    pub max_stored_filters: usize,

    /// Maximum fee history cache size.
    #[clap(long, default_value = "2048")]
    pub fee_history_limit: u64,
}
