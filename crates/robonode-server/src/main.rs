//! Main entrypoint for the Humanode's Bioauth Robonode server.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: std::net::SocketAddr = parse_env_var("ADDR")?;
    let root_filter = robonode_server::init();
    let (addr, server) =
        warp::serve(root_filter).bind_with_graceful_shutdown(addr, shutdown_signal());
    println!("Listening on http://{}", addr);
    server.await;
    Ok(())
}

/// A future that resolves when the interrupt signal is received, and panics
/// if the interrupt handler failed to set up.
async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

/// Get the value of process environment variable `key` and parse it into the type `T`.
///
/// Returns an error if the variable is not set, if the value is an invalid unicode, or if
/// the value could not be parsed.
fn parse_env_var<T>(key: &'static str) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let raw = std::env::var_os(key).ok_or_else(|| format!("{} env var is not set", key))?;
    let string = raw
        .into_string()
        .map_err(|_| format!("{} env var is not a valid unicode string", key))?;
    let v = string
        .parse()
        .map_err(|err| format!("{} env var is not valid: {}", key, err))?;
    Ok(v)
}
