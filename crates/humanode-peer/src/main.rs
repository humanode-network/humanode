//! The Humanode Peer implementation, main executable entrypoint.

mod chain_spec;
mod cli;
mod configuration;
mod qrcode;
mod rpc_url;
mod service;
mod validator_key;
mod version;

#[tokio::main]
async fn main() -> sc_cli::Result<()> {
    cli::run().await
}
