//! The "default" command implementation, used when no subcommands are provided.

use structopt::StructOpt;

use super::{params, CliConfigurationExt, SubstrateCliConfigurationProvider};

/// The `run` command used to run a node.
/// Expands the [`sc_cli::RunCmd`] with Humanode options.
#[derive(Debug, StructOpt, Clone)]
pub struct RunCmd {
    /// The base command.
    #[structopt(flatten)]
    pub base: sc_cli::RunCmd,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub bioauth_flow_params: params::BioauthFlowParams,

    /// Run the bioauth enroll flow before the authentication.
    #[structopt(long = "bioauth-enroll")]
    bioauth_perform_enroll: bool,
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
}
