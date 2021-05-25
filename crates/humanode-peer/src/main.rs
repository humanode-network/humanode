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
mod service;

#[tokio::main]
async fn main() {
    let logger = LoggerBuilder::new("");
    logger.init().unwrap();

    let mut task_manager = service::new_full(config::make()).unwrap();
    task_manager.future().await.unwrap();
    task_manager.clean_shutdown().await;
}
