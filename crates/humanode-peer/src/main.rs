//! The main entrypoint for the humanode peer.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    humanode_peer::bioauth::run().await?;
    Ok(())
}
