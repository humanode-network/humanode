//! Bioauth auth-url subcommand.

use structopt::StructOpt;

use crate::{
    cli::{params, CliConfigurationExt, SubstrateCliConfigurationProvider},
    configuration::BioauthFlow,
    qrcode::WebApp,
};

/// The `bioauth auth-url` command.
#[derive(Debug, StructOpt)]
pub struct AuthUrlCmd {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub base: Box<sc_cli::RunCmd>,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub bioauth_flow_params: params::BioauthFlowParams,
}

impl AuthUrlCmd {
    /// Run the command.
    pub fn run(&self, bioauth_flow: Option<BioauthFlow>) -> sc_cli::Result<()> {
        let (webapp_url, rpc_url) = match bioauth_flow {
            Some(BioauthFlow {
                webapp_url: Some(ref webapp_url),
                rpc_url: Some(ref rpc_url),
                ..
            }) => (webapp_url, rpc_url),
            _ => {
                return Err("bioauth flow is not configured".into());
            }
        };

        let webapp = WebApp::new(webapp_url, rpc_url)?;
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
