//! Commands hierarchy root.

use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use structopt::StructOpt;

use crate::chain_spec;

use super::{CliConfigurationExt, Runner, Subcommand};

/// The root of the CLI commands hierarchy.
#[derive(Debug, StructOpt)]
pub struct Root {
    /// Additional subcommands.
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// The `run` command used to run a node.
    #[structopt(flatten)]
    pub run: super::RunCmd,
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
        "https://link.humanode.io/bug-report".into()
    }

    fn copyright_start_year() -> i32 {
        2021
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn ChainSpec>, String> {
        k(match id {
            "dev" => Box::new(chain_spec::development_config()?),
            "" | "local" => Box::new(chain_spec::local_testnet_config()?),
            "benchmark" => Box::new(chain_spec::benchmark_config()?),
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
    pub fn create_humanode_runner<T: CliConfigurationExt>(
        &self,
        command: &T,
    ) -> sc_cli::Result<Runner<Self>> {
        // Run the init routines here; we might consider moving some of these upper in the stack.
        super::init::set_panic_handler::<Self>();
        super::init::init_logger(command)?;
        super::init::raise_fd_limit();

        Runner::new(self, command)
    }
}

impl CliConfigurationExt for sc_cli::RunCmd {}
