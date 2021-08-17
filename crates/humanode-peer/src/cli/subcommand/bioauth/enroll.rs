//! The `bioauth enroll` command.

use structopt::StructOpt;

use crate::cli::{params, CliConfigurationExt, SubstrateCliConfigurationProvider};

/// Runs the peer just like usual, but with an enroll flow enabled.
#[derive(Debug, StructOpt)]
pub struct EnrollCmd {
    /// The base command.
    #[structopt(flatten)]
    pub base: sc_cli::RunCmd,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub bioauth_flow_params: params::BioauthFlowParams,
}

impl SubstrateCliConfigurationProvider for EnrollCmd {
    type SubstrateCliConfiguration = sc_cli::RunCmd;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        &self.base
    }
}

impl CliConfigurationExt for EnrollCmd {
    fn bioauth_params(&self) -> Option<&params::BioauthFlowParams> {
        Some(&self.bioauth_flow_params)
    }

    fn bioauth_perform_enroll(&self) -> bool {
        true
    }
}
