//! The "default" command implementation, used when no subcommands are provided.

use super::{params, CliConfigurationExt, SubstrateCliConfigurationProvider};

/// The `run` command used to run a node.
/// Expands the [`sc_cli::RunCmd`] with Humanode options.
#[derive(Debug, clap::Parser, Clone)]
#[group(skip)]
pub struct RunCmd {
    /// The base command.
    #[command(flatten)]
    pub base: sc_cli::RunCmd,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[command(flatten)]
    pub bioauth_flow_params: params::BioauthFlowParams,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[command(flatten)]
    pub ethereum_rpc_params: params::EthereumRpcParams,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[command(flatten)]
    pub time_warp_params: params::TimeWarpParams,
}

impl SubstrateCliConfigurationProvider for RunCmd {
    type SubstrateCliConfiguration = sc_cli::RunCmd;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        &self.base
    }
}

impl CliConfigurationExt for RunCmd {
    fn bioauth_params(&self) -> Option<&params::BioauthFlowParams> {
        Some(&self.bioauth_flow_params)
    }

    fn ethereum_rpc_params(&self) -> Option<&params::EthereumRpcParams> {
        Some(&self.ethereum_rpc_params)
    }

    fn time_warp_params(&self) -> Option<&params::TimeWarpParams> {
        Some(&self.time_warp_params)
    }
}
