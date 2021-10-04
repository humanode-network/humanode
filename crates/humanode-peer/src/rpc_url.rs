//! The RPC URL configuration logic.

/// An RPC URL, represents the configuration parameters allowing to resolve the RPC URL.
#[derive(Debug, Clone)]
pub enum RpcUrl {
    /// The URL is not set.
    /// This can be explicit, or in case there was no explicit option and all the fallbacks failed.
    Unset,
    /// The URL is set to an explicit value.
    Set(String),
    /// The URL is to be constructed from the RPC endpoint port.
    /// This is typically used as a fallback.
    LocalhostWithPort {
        /// The port of the RPC enpoint our peer binds the socket to.
        rpc_endpoint_port: u16,
    },
    /// Detect the RPC URL from ngrok.
    DetectFromNgrok {
        /// The tunnel name to get the public URL from.
        tunnel_name: String,
    },
}

/// An RPC URL resolver provides necessary runtime components to perform RPC URL resolution.
pub struct RpcUrlResolver {
    /// The `ngrok` agent API client.
    pub ngrok_client: ngrok_api::client::Client,
}

impl Default for RpcUrlResolver {
    fn default() -> Self {
        Self {
            ngrok_client: ngrok_api::client::Client::new(
                reqwest::Client::default(),
                ngrok_api::client::Client::standard_base_url(),
            )
            .expect("standard base URL should work"),
        }
    }
}

impl RpcUrlResolver {
    /// Performs the RPC URL resolution according to the passed settings.
    /// Returns an error if the RPC URL is unset, or if we were unable to
    /// resolve it due to an error.
    pub async fn resolve(&self, val: &RpcUrl) -> Result<String, String> {
        match val {
            RpcUrl::Unset => Err("RPC URL was not set".to_owned()),
            RpcUrl::Set(url) => Ok(url.clone()),
            RpcUrl::LocalhostWithPort { rpc_endpoint_port } => {
                Ok(format!("http://localhost:{}", rpc_endpoint_port))
            }
            RpcUrl::DetectFromNgrok { tunnel_name } => {
                Ok(self.detect_from_ngrok(&*tunnel_name).await?)
            }
        }
    }

    /// Detect the RPC URL from the `ngrok`'s API.
    /// Returns the public URL from the specified tunnel.
    /// Assumes that the tunnel has a proper protocol and points to a proper port.
    async fn detect_from_ngrok(&self, tunnel_name: &str) -> Result<String, String> {
        let tunnel_name = std::borrow::Cow::Owned(tunnel_name.to_owned());
        let res = self
            .ngrok_client
            .call(&ngrok_api::data::request::TunnelInfo, (tunnel_name,))
            .await
            .map_err(|err| err.to_string())?;
        Ok(res.public_url)
    }
}
