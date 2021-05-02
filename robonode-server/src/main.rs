//! Main entrypoint for the Humanode's Bioauth Robonode server.

#![deny(missing_docs, clippy::missing_docs_in_private_items)]

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root_filter = robonode_server::init();
    let (addr, server) = warp::serve(root_filter)
        .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), shutdown_signal());
    println!("Listening on http://{}", addr);
    server.await;
    Ok(())
}

/// A future that resolves when the interrup signal is received, and panics
/// if the interrupt handler failed to set up.
async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
