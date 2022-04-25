//! Bioauth key generate subcommand logic.

use super::KeystoreBioauthPair;
use bip39::{Language, Mnemonic, MnemonicType};
use sc_cli::{utils, OutputTypeFlag};
use structopt::StructOpt;

/// The `bioauth key generate` command.
#[derive(Debug, StructOpt)]
pub struct GenerateKeyCmd {
    /// The number of words in the phrase to generate. One of 12 (default), 15, 18, 21 and 24.
    #[structopt(long)]
    words: Option<usize>,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub output_scheme: OutputTypeFlag,
}

/// An error that occured during key generation.
#[derive(thiserror::Error, Debug)]
pub enum GenerateKeyError {
    /// The number of words used in mnemonic is invalid.
    #[error("Invalid number of words given for phrase: must be 12/15/18/21/24.")]
    InvalidNumberOfWords,
}

impl GenerateKeyCmd {
    /// Run the generate command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let words = match self.words {
            Some(words) => MnemonicType::for_word_count(words).map_err(|_| {
                sc_cli::Error::Input((GenerateKeyError::InvalidNumberOfWords).to_string())
            })?,
            None => MnemonicType::Words12,
        };
        let mnemonic = Mnemonic::new(words, Language::English);
        let output = self.output_scheme.output_type;

        // Password is None as we don't use it for keystore at the current moment.
        // Network_override is None as we don't allow to override network type as
        // the subcommand is used for Bioauth network explicitly.
        utils::print_from_uri::<KeystoreBioauthPair>(mnemonic.phrase(), None, None, output);

        Ok(())
    }
}
