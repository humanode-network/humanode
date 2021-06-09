//! The Humanode Peer implementation, main executable entrypoint.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use sc_tracing::logging::LoggerBuilder;

mod chain_spec;
mod config;
mod dummy;
mod rpc;
mod service;

#[tokio::main]
async fn main() {
    let logger = LoggerBuilder::new("");
    logger.init().unwrap();

    let mut task_manager = service::new_full(config::make()).unwrap();

    tokio::select! {
        res = task_manager.future() => res.unwrap(),
        res = tokio::signal::ctrl_c() => {
            res.unwrap();
            tracing::info!("Got Ctrl+C");
        }
    }
    task_manager.clean_shutdown().await;
}
