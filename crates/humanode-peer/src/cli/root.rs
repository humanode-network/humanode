//! Commands hierarchy root.

use sc_cli::{ChainSpec, CliConfiguration, RuntimeVersion, SubstrateCli};
use structopt::StructOpt;

use crate::chain_spec;

use super::{runner::Runner, subcommand::Subcommand};

/// The root of the CLI commands hierarchy.
#[derive(Debug, StructOpt)]
pub struct Root {
    /// Additional subcommands.
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// The `run` command used to run a node.
    #[structopt(flatten)]
    pub run: sc_cli::RunCmd,
}

impl SubstrateCli for Root {
    fn impl_name() -> String {
        "Humanode Node".into()
    }

    fn impl_version() -> String {
        "0".to_owned()
    }

    fn description() -> String {
        "Biologically verified human-nodes as a basis for a fair financial system.".into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/humanode-network/humanode/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2021
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::development_config()?),
            "" | "local" => Box::new(chain_spec::local_testnet_config()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }

    fn native_runtime_version(_chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &humanode_runtime::VERSION
    }
}

impl Root {
    /// Create a [`Runner`] for the command provided in argument.
    /// This will create a [`Configuration`] from the command line arguments and the rest of
    /// the environemnt.
    pub fn create_humanode_runner<T: CliConfiguration>(
        &self,
        command: &T,
    ) -> sc_cli::Result<Runner<Self>> {
        command.init::<Self>()?;
        Runner::new(self, command)
    }
}
