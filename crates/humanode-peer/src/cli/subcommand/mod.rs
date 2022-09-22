//! Humanode peer subcommands.
//! The `substrate` built-in commands are embedded as-is, additional commands are introduced as
//! nested `mod`s in this `mod`.

use super::CliConfigurationExt;

pub mod bioauth;
pub mod ethereum;

/// Humanode peer subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management cli utilities
    #[clap(subcommand)]
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
    #[clap(subcommand)]
    Bioauth(bioauth::BioauthCmd),

    /// Ethereum related subcommands.
    #[clap(subcommand)]
    Ethereum(ethereum::EthereumCmd),

    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[clap(name = "benchmark", about = "Benchmark runtime pallets.")]
    #[clap(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Db meta columns information.
    FrontierDb(fc_cli::FrontierDbCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
    #[cfg(not(feature = "try-runtime"))]
    TryRuntime,
}

impl CliConfigurationExt for sc_cli::BuildSpecCmd {}
impl CliConfigurationExt for sc_cli::CheckBlockCmd {}
impl CliConfigurationExt for sc_cli::ExportBlocksCmd {}
impl CliConfigurationExt for sc_cli::ExportStateCmd {}
impl CliConfigurationExt for sc_cli::ImportBlocksCmd {}
impl CliConfigurationExt for sc_cli::PurgeChainCmd {}
impl CliConfigurationExt for sc_cli::RevertCmd {}
impl CliConfigurationExt for frame_benchmarking_cli::BenchmarkCmd {}
impl CliConfigurationExt for fc_cli::FrontierDbCmd {}
#[cfg(feature = "try-runtime")]
impl CliConfigurationExt for try_runtime_cli::TryRuntimeCmd {}
