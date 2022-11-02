//! The Humanode Peer implementation, main executable entrypoint.

mod api_versions;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod build_info;
mod chain_spec;
mod cli;
mod configuration;
mod qrcode;
mod rpc_url;
mod service;
mod time_warp;
mod validator_key;

#[tokio::main]
async fn main() -> sc_cli::Result<()> {
    cli::run().await
}
