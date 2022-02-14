//! Ethereum subcommands and related common utilities.

use structopt::StructOpt;

pub mod generate;
pub mod inspect;
pub mod utils;

/// Subcommands for the `ethereum` command.
#[derive(Debug, StructOpt)]
pub enum EthereumCmd {
    /// Generate a random account.
    GenerateAccount(generate::GenerateAccountCmd),
    /// Inspect a provided mnemonic.
    InspectAccount(inspect::InspectAccountCmd),
}
