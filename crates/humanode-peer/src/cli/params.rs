//! Shared CLI parameters.

use structopt::StructOpt;

/// Shared CLI parameters used to configure bioauth flow.
#[derive(Debug, StructOpt, Clone)]
pub struct BioauthFlowParams {
    /// The URL to use for the web app.
    /// Used to print the QR Code to the console, so it doesn't matter much.
    #[structopt(long, value_name = "WEBAPP_URL")]
    pub webapp_url: Option<String>,

    /// The URL to pass to the web app to connect to the node RPC.
    /// If not passed, a URL with `localhost` and the HTTP RPC port will be used.
    #[structopt(long, value_name = "RPC_URL", conflicts_with_all = &["rpc-url-ngrok-detect", "rpc-url-unset"])]
    pub rpc_url: Option<String>,

    /// Detect RPC URL from ngrok.
    #[structopt(long, conflicts_with_all = &["rpc-url", "rpc-url-unset"])]
    pub rpc_url_ngrok_detect: bool,

    /// Explicitly unset the RPC URL.
    #[structopt(long, conflicts_with_all = &["rpc-url", "rpc-url-ngrok-detect"])]
    pub rpc_url_unset: bool,

    /// The tunnel name at ngrok to detect RPC URL from, if ngrok is used to detect the RPC URL.
    #[structopt(long, value_name = "TUNNEL_NAME", default_value = "command_line")]
    pub rpc_url_ngrok_detect_from: String,

    /// The URL of robonode to authenticate with.
    #[structopt(long, value_name = "ROBONODE_URL")]
    pub robonode_url: Option<String>,
}

/// Shared CLI parameters used to configure evm.
#[derive(Debug, StructOpt, Clone)]
pub struct EvmParams {
    /// Maximum number of logs in a query.
    #[structopt(long, default_value = "10000")]
    pub max_past_logs: u32,

    /// Maximum number of stored filters.
    #[structopt(long, default_value = "500")]
    pub max_stored_filters: usize,

    /// The dynamic-fee pallet target gas price set by block author.
    #[structopt(long, default_value = "1")]
    pub target_gas_price: u64,
}
