//! Humanode peer commands.

use super::substrate_cmd_adapter;

pub mod bioauth;
pub mod ethereum;
pub mod run;

/// The root command.
#[derive(Debug, clap::Parser)]
pub struct Command {
    /// Additional subcommands.
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// The `run` command used to run a node.
    #[clap(flatten)]
    pub run: run::RunCmd,
}

/// The subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management cli utilities.
    // The key subcommand is a special snowflake, because it doesn't implement the Substrate CLI
    // config.
    #[clap(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(
        substrate_cmd_adapter::Cmd<sc_cli::BuildSpecCmd, substrate_cmd_adapter::NoExtraParams>,
    ),

    /// Validate blocks.
    CheckBlock(
        substrate_cmd_adapter::Cmd<sc_cli::CheckBlockCmd, substrate_cmd_adapter::NoExtraParams>,
    ),

    /// Export blocks.
    ExportBlocks(
        substrate_cmd_adapter::Cmd<sc_cli::ExportBlocksCmd, substrate_cmd_adapter::NoExtraParams>,
    ),

    /// Export the state of a given block into a chain spec.
    ExportState(
        substrate_cmd_adapter::Cmd<sc_cli::ExportStateCmd, substrate_cmd_adapter::NoExtraParams>,
    ),

    /// Import blocks.
    ImportBlocks(
        substrate_cmd_adapter::Cmd<sc_cli::ImportBlocksCmd, substrate_cmd_adapter::NoExtraParams>,
    ),

    /// Remove the whole chain.
    PurgeChain(
        substrate_cmd_adapter::Cmd<sc_cli::PurgeChainCmd, substrate_cmd_adapter::NoExtraParams>,
    ),

    /// Revert the chain to a previous state.
    Revert(substrate_cmd_adapter::Cmd<sc_cli::RevertCmd, substrate_cmd_adapter::NoExtraParams>),

    /// Biometric authentication related subcommands.
    #[clap(subcommand)]
    Bioauth(bioauth::BioauthCmd),

    /// Ethereum related subcommands.
    #[clap(subcommand)]
    Ethereum(ethereum::EthereumCmd),

    /// Benchmarking utilities.
    #[clap(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}
