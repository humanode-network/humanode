//! Print the resolved RPC URL.

use crate::{
    cli::{
        params, utils::application_error, CliConfigurationExt, SubstrateCliConfigurationProvider,
    },
    configuration::BioauthFlow,
};

/// The `rpc-url` command.
#[derive(Debug, clap::Parser)]
pub struct RpcUrlCmd {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[command(flatten)]
    pub base: Box<sc_cli::RunCmd>,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[command(flatten)]
    pub bioauth_flow_params: params::BioauthFlowParams,
}

impl RpcUrlCmd {
    /// Resolve the RPC URL the same way the node does, and print it.
    ///
    /// Explicit `--rpc-url`, localhost-from-port, scheme preference, and ngrok detection all go
    /// through [`crate::rpc_url::RpcUrlResolver`]. A webapp URL is not required.
    pub async fn run(&self, bioauth_flow: Option<BioauthFlow>) -> sc_cli::Result<()> {
        let bioauth_flow = bioauth_flow
            .ok_or("bioauth flow is not configured")
            .map_err(application_error)?;
        let rpc_url = bioauth_flow
            .rpc_url_resolver
            .resolve(&bioauth_flow.rpc_url)
            .await
            .map_err(application_error)?;
        println!("{rpc_url}");
        Ok(())
    }
}

impl SubstrateCliConfigurationProvider for RpcUrlCmd {
    type SubstrateCliConfiguration = sc_cli::RunCmd;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        &self.base
    }
}

impl CliConfigurationExt for RpcUrlCmd {
    fn bioauth_params(&self) -> Option<&params::BioauthFlowParams> {
        Some(&self.bioauth_flow_params)
    }
}
