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
        /// The port of the RPC endpoint our peer binds the socket to.
        rpc_endpoint_port: u16,
        /// The scheme to use for the RPC URL.
        scheme: &'static str,
    },
    /// Detect the RPC URL from ngrok.
    DetectFromNgrok {
        /// The tunnel name to get the public URL from.
        tunnel_name: String,
        /// The WebSocket port to match against, and switch protocol to WebSocket if the tunnel
        /// address has this port.
        ws_rpc_endpoint_port: Option<u16>,
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
            RpcUrl::LocalhostWithPort {
                rpc_endpoint_port,
                scheme,
            } => Ok(format!("{}://localhost:{}", scheme, rpc_endpoint_port).into()),
            RpcUrl::DetectFromNgrok {
                tunnel_name,
                ws_rpc_endpoint_port,
            } => Ok(self
                .detect_from_ngrok(&*tunnel_name, *ws_rpc_endpoint_port)
                .await?
                .into()),
        }
    }

    /// Detect the RPC URL from the `ngrok`'s API.
    /// Returns the public URL from the specified tunnel.
    /// Assumes that the tunnel has a proper protocol and points to a proper port.
    async fn detect_from_ngrok(
        &self,
        tunnel_name: &str,
        ws_rpc_endpoint_port: Option<u16>,
    ) -> Result<String, String> {
        let mut attempts_left = 100;
        let res = loop {
            let tunnel_name = std::borrow::Cow::Owned(tunnel_name.to_owned());
            let result = self
                .ngrok_client
                .call(&ngrok_api::data::request::TunnelInfo, (tunnel_name,))
                .await;

            let err = match result {
                Ok(res) => break res,
                Err(err) => match err {
                    ngrok_api::client::Error::BadStatus(status) if status == 404 => err,
                    ngrok_api::client::Error::Reqwest(ref reqwest_error)
                        if reqwest_error.is_redirect()
                            || reqwest_error.is_status()
                            || reqwest_error.is_timeout()
                            || reqwest_error.is_request()
                            || reqwest_error.is_connect()
                            || reqwest_error.is_body()
                            || reqwest_error.is_decode() =>
                    {
                        err
                    }
                    err => return Err(format!("unable to detect the RPC URL from ngrok: {}", err)),
                },
            };

            if attempts_left <= 0 {
                return Err(format!("ngrok did not start the tunnel in time: {}", err));
            }
            attempts_left -= 1;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        };
        let mut public_url = res.public_url;
        if let Some(ws_rpc_endpoint_port) = ws_rpc_endpoint_port {
            if res
                .config
                .addr
                .ends_with(&format!(":{}", ws_rpc_endpoint_port))
            {
                public_url = public_url.replacen("https", "wss", 1);
            }
        }
        Ok(public_url)
    }
}
