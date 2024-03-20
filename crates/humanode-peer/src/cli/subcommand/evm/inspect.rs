//! Ethereum inspect subcommand.

use bip39::{Language, Mnemonic};

use super::utils::extract_and_print_keys;

/// The mnemonic input.
#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct MnemonicInput {
    /// Specify the mnemonic.
    #[arg(long, short = 'm')]
    mnemonic: Option<String>,

    /// Use the built-in dev mnemonic.
    #[arg(long, short = 'd')]
    dev: bool,
}

/// The `ethereum inspect` command.
#[derive(Debug, clap::Parser)]
pub struct InspectAccountCmd {
    /// The mnemonic input.
    #[command(flatten)]
    pub mnemonic_input: MnemonicInput,

    /// The account index to use in the derivation path.
    #[arg(long = "account-index", short = 'a')]
    account_index: Option<u32>,
}

impl InspectAccountCmd {
    /// Run the inspect command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let mnemonic = self.mnemonic_input.mnemonic.as_ref();

        let mnemonic = mnemonic
            .map(|mnemonic| Mnemonic::from_phrase(mnemonic, Language::English))
            .transpose()
            .map_err(|err| sc_cli::Error::Input(err.to_string()))?;

        extract_and_print_keys(mnemonic.as_ref(), self.account_index)
            .map_err(|err| sc_cli::Error::Application(Box::new(err)))?;

        Ok(())
    }
}
