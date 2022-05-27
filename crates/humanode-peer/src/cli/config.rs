//! Machinery to populate the configuration from the CLI arguments.

use sc_chain_spec::get_extension;

use super::{params, BioauthFlowParams, RpcUrlSchemePreference};
use crate::{
    chain_spec::Extensions,
    configuration::{self, Configuration},
    rpc_url::RpcUrl,
};

/// An extension to the [`sc_cli::CliConfiguration`] to enable us to pass custom params.
pub trait CliConfigurationExt: SubstrateCliConfigurationProvider {
    /// Create a [`Configuration`] object from the CLI params.
    fn create_humanode_configuration<C: sc_cli::SubstrateCli>(
        &self,
        cli: &C,
        tokio_handle: tokio::runtime::Handle,
    ) -> sc_cli::Result<Configuration> {
        let substrate = sc_cli::CliConfiguration::create_configuration(
            self.substrate_cli_configuration(),
            cli,
            tokio_handle,
        )?;

        let extensions = get_extension::<Extensions>(substrate.chain_spec.extensions())
            .cloned()
            .unwrap_or_default();

        let bioauth_flow = self.bioauth_params().map(|params| {
            let rpc_http_port = substrate.rpc_http.map(|v| v.port());
            let rpc_ws_port = substrate.rpc_ws.map(|v| v.port());
            let rpc_url = rpc_url_from_params(params, rpc_http_port, rpc_ws_port);

            configuration::BioauthFlow {
                rpc_url_resolver: Default::default(),
                robonode_url: params
                    .robonode_url
                    .clone()
                    .or(extensions.robonode_url)
                    .unwrap_or_else(|| "http://127.0.0.1:3033".into()),
                webapp_url: params.webapp_url.clone().or(extensions.webapp_url),
                rpc_url,
            }
        });

        let evm = self.evm_params().map(|params| configuration::Evm {
            max_past_logs: params.max_past_logs,
            max_stored_filters: params.max_stored_filters,
            target_gas_price: params.target_gas_price,
            fee_history_limit: params.fee_history_limit,
        });

        Ok(Configuration {
            substrate,
            bioauth_flow,
            evm,
        })
    }

    /// Provide the bioauth flow params, if available.
    fn bioauth_params(&self) -> Option<&params::BioauthFlowParams> {
        None
    }

    /// Provide the evm params, if available.
    fn evm_params(&self) -> Option<&params::EvmParams> {
        None
    }
}

/// Indirect relation to the [`sc_cli::CliConfiguration`] for any type.
pub trait SubstrateCliConfigurationProvider {
    /// A type providing the [`sc_cli::CliConfiguration`].
    type SubstrateCliConfiguration: sc_cli::CliConfiguration;

    /// Obtain the [`sc_cli::CliConfiguration`] implementation.
    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration;
}

impl<T: sc_cli::CliConfiguration> SubstrateCliConfigurationProvider for T {
    type SubstrateCliConfiguration = T;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        self
    }
}

/// Construct an RPC URL from the bioauth flow params and an RPC endpoint port.
fn rpc_url_from_params(
    params: &BioauthFlowParams,
    rpc_http_port: Option<u16>,
    rpc_ws_port: Option<u16>,
) -> RpcUrl {
    if let Some(val) = &params.rpc_url {
        return RpcUrl::Set(val.clone());
    }
    if params.rpc_url_unset {
        return RpcUrl::Unset;
    }
    if params.rpc_url_ngrok_detect {
        let ws_rpc_endpoint_port = match params.rpc_url_scheme_preference {
            // If there's no preference - try switching to WebSocket if it's available.
            RpcUrlSchemePreference::NoPreference | RpcUrlSchemePreference::Ws => rpc_ws_port,
            RpcUrlSchemePreference::Http => None,
        };
        return RpcUrl::DetectFromNgrok {
            tunnel_name: params.rpc_url_ngrok_detect_from.clone(),
            ws_rpc_endpoint_port,
        };
    }

    match (
        &params.rpc_url_scheme_preference,
        rpc_http_port,
        rpc_ws_port,
    ) {
        // Try WebSocket first if the user has no preference.
        (RpcUrlSchemePreference::Ws | RpcUrlSchemePreference::NoPreference, _, Some(port)) => {
            RpcUrl::LocalhostWithPort {
                rpc_endpoint_port: port,
                scheme: "ws",
            }
        }
        // Try HTTP second if the user has no preference.
        (RpcUrlSchemePreference::Http | RpcUrlSchemePreference::NoPreference, Some(port), _) => {
            RpcUrl::LocalhostWithPort {
                rpc_endpoint_port: port,
                scheme: "http",
            }
        }
        // If everything fails - fallback to unset.
        _ => RpcUrl::Unset,
    }
}
