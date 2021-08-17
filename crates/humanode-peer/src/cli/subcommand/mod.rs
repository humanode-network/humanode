//! Humanode peer subcommands.
//! The `substrate` built-in commands are embedded as-is, additional commands are introduced as
//! nested `mod`s in this `mod`.

use structopt::StructOpt;

use super::CliConfigurationExt;

pub mod bioauth;

/// Humanode peer subcommands.
#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Key management cli utilities
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Biometric authentication related subcommands.
    Bioauth(Box<bioauth::BioauthCmd>),

    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}

impl CliConfigurationExt for sc_cli::BuildSpecCmd {}
impl CliConfigurationExt for sc_cli::CheckBlockCmd {}
impl CliConfigurationExt for sc_cli::ExportBlocksCmd {}
impl CliConfigurationExt for sc_cli::ExportStateCmd {}
impl CliConfigurationExt for sc_cli::ImportBlocksCmd {}
impl CliConfigurationExt for sc_cli::PurgeChainCmd {}
impl CliConfigurationExt for sc_cli::RevertCmd {}
impl CliConfigurationExt for frame_benchmarking_cli::BenchmarkCmd {}
