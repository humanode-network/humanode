//! Ethereum subcommands and related common utilities.

pub mod generate;
pub mod inspect;
pub mod utils;

/// Subcommands for the `ethereum` command.
#[derive(Debug, clap::Subcommand)]
pub enum EvmCmd {
    /// Generate a random account.
    GenerateAccount(generate::GenerateAccountCmd),
    /// Inspect a provided mnemonic.
    InspectAccount(inspect::InspectAccountCmd),
}

impl EvmCmd {
    /// Run the ethereum subcommands
    pub async fn run(&self) -> sc_cli::Result<()> {
        match self {
            EvmCmd::GenerateAccount(cmd) => cmd.run().await,
            EvmCmd::InspectAccount(cmd) => cmd.run().await,
        }
    }
}
