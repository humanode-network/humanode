//! The "default" command implementation, used when no subcommands are provided.

use structopt::StructOpt;

use super::{CliConfigurationExt, SubstrateCliConfigurationProvider};

/// The `run` command used to run a node.
/// Expands the [`sc_cli::RunCmd`] with Humanode options.
#[derive(Debug, StructOpt, Clone)]
pub struct RunCmd {
    /// The base command.
    #[structopt(flatten)]
    pub base: sc_cli::RunCmd,
}

impl SubstrateCliConfigurationProvider for RunCmd {
    type SubstrateCliConfiguration = sc_cli::RunCmd;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        &self.base
    }
}

impl CliConfigurationExt for RunCmd {}
