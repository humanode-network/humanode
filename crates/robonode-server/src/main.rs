//! Main entrypoint for the Humanode's Bioauth Robonode server.

use std::env::VarError;

use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut logger = sc_tracing::logging::LoggerBuilder::new(parse_log_level());
    logger.with_colors(true);
    logger.init()?;

    let addr: std::net::SocketAddr = env("ADDR")?;
    let facetec_server_url = env("FACETEC_SERVER_URL")?;
    let facetec_device_key_identifier: String = env("FACETEC_DEVICE_KEY_IDENTIFIER")?;
    let facetec_public_face_map_encryption_key = env("FACETEC_PUBLIC_FACE_MAP_ENCRYPTION_KEY")?;
    let facetec_production_key: Option<String> = maybe_env("FACETEC_PRODUCTION_KEY")?;
    let robonode_keypair_string: String = env("ROBONODE_KEYPAIR")?;
    let mut robonode_keypair_bytes: [u8; 64] = [0; 64];
    hex::decode_to_slice(robonode_keypair_string, &mut robonode_keypair_bytes)?;
    let robonode_keypair = robonode_crypto::Keypair::from_keypair_bytes(&robonode_keypair_bytes)?;

    let facetec_api_client = facetec_api_client::Client {
        base_url: facetec_server_url,
        reqwest: reqwest::Client::new(),
        device_key_identifier: facetec_device_key_identifier.clone(),
        injected_ip_address: None,
        response_body_error_inspector: robonode_server::LoggingInspector,
    };
    let face_tec_device_sdk_params = robonode_server::FacetecDeviceSdkParams {
        device_key_identifier: facetec_device_key_identifier,
        public_face_map_encryption_key: facetec_public_face_map_encryption_key,
        production_key: facetec_production_key,
    };

    let execution_id = uuid::Uuid::new_v4();

    let root_filter = robonode_server::init(
        execution_id,
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
fn env<T>(key: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let val = maybe_env(key)?;
    let val = val.ok_or_else(|| format!("env variable {key} is not set"))?;
    Ok(val)
}

/// Get the value of process environment variable `key` and parse it into the type `T` if variable is set.
///
/// Returns an error if the value is an invalid unicode or if the value could not be parsed.
fn maybe_env<T>(key: &str) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let val = match std::env::var(key) {
        Ok(val) => val,
        Err(VarError::NotPresent) => return Ok(None),
        Err(VarError::NotUnicode(err)) => {
            format!("{key} env var is not a valid unicode string: {err:?}")
        }
    };
    let val = val
        .parse()
        .map_err(|err| format!("{key} env var is not valid: {err}"))?;
    Ok(Some(val))
}

/// Parse log level from the env vars.
fn parse_log_level() -> String {
    let maybe_level: Result<String, _> = env("RUST_LOG");
    let maybe_level: Result<String, _> = maybe_level.or_else(|_| env("LOG"));
    maybe_level.unwrap_or_else(|_| "debug".into())
}
