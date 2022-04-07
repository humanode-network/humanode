//! Bioauth key inspect subcommand logic.

use super::BioauthPair;
use bip39::{Language, Mnemonic};
use sc_cli::{utils, OutputTypeFlag};
use structopt::StructOpt;

/// The `bioauth key generate` command.
#[derive(Debug, StructOpt)]
pub struct InspectKeyCmd {
    /// The secret key uri (mnemonic).
    #[structopt(long, short = "m")]
    suri: String,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub output_scheme: OutputTypeFlag,
}

impl InspectKeyCmd {
    /// Run the generate command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let mnemonic = Mnemonic::from_phrase(&self.suri, Language::English)
            .map_err(|err| sc_cli::Error::Input(err.to_string()))?;
        let output = self.output_scheme.output_type;

        // Password is None as we don't use it for keystore at the current moment.
        // Network_override is None as we don't allow to override network type as
        // the subcommand is used for Bioauth network explicitly.
        utils::print_from_uri::<BioauthPair>(mnemonic.phrase(), None, None, output);

        Ok(())
    }
}
