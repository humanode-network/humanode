//! Ethereum generate subcommand.

use bip39::{Language, Mnemonic, MnemonicType};
use structopt::StructOpt;

use super::utils::extract_and_print_keys;

/// The `ethereum generate` command.
#[derive(Debug, StructOpt)]
pub struct GenerateAccountCmd {
    /// Generate 24 words mnemonic instead of 12.
    #[structopt(long, short = "w")]
    w24: bool,

    /// The account index to use in the derivation path.
    #[structopt(long = "account-index", short = "a")]
    account_index: Option<u32>,
}

impl GenerateAccountCmd {
    /// Run the generate command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let mnemonic = match self.w24 {
            true => Mnemonic::new(MnemonicType::Words24, Language::English),
            false => Mnemonic::new(MnemonicType::Words12, Language::English),
        };

        extract_and_print_keys(&mnemonic, self.account_index)?;

        Ok(())
    }
}
