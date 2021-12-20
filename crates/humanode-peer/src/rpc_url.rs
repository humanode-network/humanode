//! The RPC URL configuration logic.

use std::borrow::Cow;

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
    pub async fn resolve<'a>(&self, val: &'a RpcUrl) -> Result<Cow<'a, str>, Cow<'static, str>> {
        match val {
            RpcUrl::Unset => Err(Cow::Borrowed("RPC URL was not set")),
            RpcUrl::Set(url) => Ok(Cow::Borrowed(url)),
            RpcUrl::LocalhostWithPort { rpc_endpoint_port } => {
                Ok(format!("http://localhost:{}", rpc_endpoint_port).into())
            }
            RpcUrl::DetectFromNgrok { tunnel_name } => {
                Ok(self.detect_from_ngrok(&*tunnel_name).await?.into())
            }
        }
    }

    /// Detect the RPC URL from the `ngrok`'s API.
    /// Returns the public URL from the specified tunnel.
    /// Assumes that the tunnel has a proper protocol and points to a proper port.
    async fn detect_from_ngrok(&self, tunnel_name: &str) -> Result<String, String> {
        let mut attempts_left = 10;
        let res = loop {
            let tunnel_name = std::borrow::Cow::Owned(tunnel_name.to_owned());
            let result = self
                .ngrok_client
                .call(&ngrok_api::data::request::TunnelInfo, (tunnel_name,))
                .await;
            match result {
                Ok(res) => break res,
                Err(ngrok_api::client::Error::BadStatus(status)) if status == 404 => {
                    if attempts_left <= 0 {
                        return Err("ngrok did not start the tunnel in time".to_owned());
                    }
                    attempts_left -= 1;
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                Err(err) => return Err(err.to_string()),
            }
        };
        Ok(res.public_url)
    }
}
