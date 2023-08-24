//! Ethereum inspect subcommand.

use bip39::{Language, Mnemonic};

use super::utils::extract_and_print_keys;

/// The `ethereum inspect` command.
#[derive(Debug, clap::Parser)]
pub struct InspectAccountCmd {
    /// Specify the mnemonic.
    #[arg(long, short = 'm')]
    mnemonic: Option<String>,

    /// The account index to use in the derivation path.
    #[arg(long = "account-index", short = 'a')]
    account_index: Option<u32>,
}

impl InspectAccountCmd {
    /// Run the inspect command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let mnemonic = self
            .mnemonic
            .as_ref()
            .map(|mnemonic| {
                Mnemonic::from_phrase(mnemonic, Language::English)
                    .map_err(|err| sc_cli::Error::Input(err.to_string()))
            })
            .transpose()?;

        extract_and_print_keys(mnemonic.as_ref(), self.account_index)
            .map_err(|err| sc_cli::Error::Application(Box::new(err)))?;

        Ok(())
    }
}
