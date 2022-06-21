//! Bioauth auth-url subcommand.

use crate::{
    cli::{
        params, utils::application_error, CliConfigurationExt, SubstrateCliConfigurationProvider,
    },
    configuration::BioauthFlow,
    qrcode::WebApp,
};

/// The `bioauth auth-url` command.
#[derive(Debug, clap::Parser)]
pub struct AuthUrlCmd {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[clap(flatten)]
    pub base: Box<sc_cli::RunCmd>,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[clap(flatten)]
    pub bioauth_flow_params: params::BioauthFlowParams,
}

impl AuthUrlCmd {
    /// Run the command.
    pub async fn run(&self, bioauth_flow: Option<BioauthFlow>) -> sc_cli::Result<()> {
        let bioauth_flow = bioauth_flow
            .ok_or("bioauth flow is not configured")
            .map_err(application_error)?;
        let (webapp_url, rpc_url) = bioauth_flow
            .qrcode_params()
            .await
            .map_err(application_error)?;

        let webapp = WebApp::new(webapp_url, rpc_url).map_err(application_error)?;
        println!("{}", webapp.url());
        Ok(())
    }
}

impl SubstrateCliConfigurationProvider for AuthUrlCmd {
    type SubstrateCliConfiguration = sc_cli::RunCmd;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        &self.base
    }
}

impl CliConfigurationExt for AuthUrlCmd {
    fn bioauth_params(&self) -> Option<&params::BioauthFlowParams> {
        Some(&self.bioauth_flow_params)
    }
}
