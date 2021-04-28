//! The main entrypoint for the humanode peer.

#![deny(missing_docs, clippy::missing_docs_in_private_items)]

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    humanode_peer::bioauth::run().await?;
    Ok(())
}
