//! Ethereum inspect subcommand.

use bip39::{Language, Mnemonic};

use super::utils::extract_and_print_keys;

/// The `ethereum inspect` command.
#[derive(Debug, clap::Parser)]
pub struct InspectAccountCmd {
    /// Specify the mnemonic.
    #[arg(long, short = 'm')]
    mnemonic: String,

    /// The account index to use in the derivation path.
    #[arg(long = "account-index", short = 'a')]
    account_index: Option<u32>,
}

impl InspectAccountCmd {
    /// Run the inspect command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let mnemonic = Mnemonic::from_phrase(&self.mnemonic, Language::English)
            .map_err(|err| sc_cli::Error::Input(err.to_string()))?;

        extract_and_print_keys(&mnemonic, self.account_index)?;

        Ok(())
    }
}
