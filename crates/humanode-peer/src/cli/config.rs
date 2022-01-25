//! Machinery to populate the configuration from the CLI arguments.

use sc_chain_spec::get_extension;

use crate::{
    chain_spec::Extensions,
    configuration::{self, Configuration},
    rpc_url::RpcUrl,
};

use super::{params, BioauthFlowParams};

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
            let rpc_port = substrate.rpc_http.map(|v| v.port());
            let rpc_url = rpc_url_from_params(params, rpc_port);

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

        let evm = if self.is_full_node_run() {
            let evm_params = self.evm_params();
            configuration::Evm {
                target_gas_price: evm_params.target_gas_price,
            }
        } else {
            Default::default()
        };

        let evm_rpc = self.evm_rpc_params().map(|params| configuration::EvmRpc {
            max_past_logs: params.max_past_logs,
            max_stored_filters: params.max_stored_filters,
            fee_history_limit: params.fee_history_limit,
        });

        Ok(Configuration {
            substrate,
            bioauth_flow,
            bioauth_perform_enroll: self.bioauth_perform_enroll(),
            evm,
            evm_rpc,
        })
    }

    /// Provide the bioauth flow params, if available.
    fn bioauth_params(&self) -> Option<&params::BioauthFlowParams> {
        None
    }

    /// Whether to perform the bioauth enroll before the authentication or not.
    fn bioauth_perform_enroll(&self) -> bool {
        false
    }

    /// Provide the evm params.
    fn evm_params(&self) -> params::EvmParams {
        <params::EvmParams as Default>::default()
    }

    /// Provide the evm rpc params, if available.
    fn evm_rpc_params(&self) -> Option<&params::EvmRpcParams> {
        None
    }

    /// Whether a full node run or not.
    fn is_full_node_run(&self) -> bool {
        false
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
fn rpc_url_from_params(params: &BioauthFlowParams, rpc_port: Option<u16>) -> RpcUrl {
    if let Some(val) = &params.rpc_url {
        return RpcUrl::Set(val.clone());
    }
    if params.rpc_url_unset {
        return RpcUrl::Unset;
    }
    if params.rpc_url_ngrok_detect {
        return RpcUrl::DetectFromNgrok {
            tunnel_name: params.rpc_url_ngrok_detect_from.clone(),
        };
    }

    if let Some(rpc_endpoint_port) = rpc_port {
        return RpcUrl::LocalhostWithPort { rpc_endpoint_port };
    }

    RpcUrl::Unset
}
