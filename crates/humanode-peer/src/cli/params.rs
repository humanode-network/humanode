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

    /// The tunnel name at ngrok to detext RPC URL from, if ngrok is used to detect the RPC URL.
    #[structopt(long, value_name = "TUNNEL_NAME", default_value = "command_line")]
    pub rpc_url_ngrok_detect_from: String,

    /// The URL of robonode to authenticate with.
    #[structopt(long, value_name = "ROBONODE_URL")]
    pub robonode_url: Option<String>,
}
