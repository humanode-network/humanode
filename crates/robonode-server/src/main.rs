//! Main entrypoint for the Humanode's Bioauth Robonode server.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut logger = sc_tracing::logging::LoggerBuilder::new("debug");
    logger.with_colors(true);
    let _ = logger.init()?;

    let addr: std::net::SocketAddr = parse_env_var("ADDR")?;
    let facetec_server_url = parse_env_var("FACETEC_SERVER_URL")?;
    let facetec_device_key_identifier: String = parse_env_var("FACETEC_DEVICE_KEY_IDENTIFIER")?;
    let facetec_public_face_map_encryption_key =
        parse_env_var("FACETEC_PUBLIC_FACE_MAP_ENCRYPTION_KEY")?;
    let robonode_keypair_string: String = parse_env_var("ROBONODE_KEYPAIR")?;
    let robonode_keypair_bytes = hex::decode(robonode_keypair_string)?;
    let robonode_keypair = robonode_crypto::Keypair::from_bytes(robonode_keypair_bytes.as_slice())?;

    let facetec_api_client = facetec_api_client::Client {
        base_url: facetec_server_url,
        reqwest: reqwest::Client::new(),
        device_key_identifier: facetec_device_key_identifier.clone(),
        response_body_error_inspector: facetec_api_client::response_body_error::NoopInspector,
    };
    let face_tec_device_sdk_params = robonode_server::FacetecDeviceSdkParams {
        device_key_identifier: facetec_device_key_identifier,
        public_face_map_encryption_key: facetec_public_face_map_encryption_key,
    };

    let root_filter = robonode_server::init(
        facetec_api_client,
        face_tec_device_sdk_params,
        robonode_keypair,
    );
    let (addr, server) =
        warp::serve(root_filter).bind_with_graceful_shutdown(addr, shutdown_signal());

    info!("Listening on http://{}", addr);

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
