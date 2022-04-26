//! Bioauth key list subcommand logic.

use super::KeystoreBioauthId;
use sc_cli::{CliConfiguration, KeystoreParams, SharedParams};
use sc_service::KeystoreContainer;
use structopt::StructOpt;

use crate::cli::CliConfigurationExt;

/// The `bioauth key list` command.
#[derive(Debug, StructOpt)]
pub struct ListKeysCmd {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub shared_params: SharedParams,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub keystore_params: KeystoreParams,
}

impl ListKeysCmd {
    /// Run the list command.
    pub async fn run(&self, keystore_container: KeystoreContainer) -> sc_cli::Result<()> {
        let keystore = keystore_container.keystore();
        let keys =
            crate::validator_key::AppCryptoPublic::<KeystoreBioauthId>::list(keystore.as_ref())
                .await
                .map_err(|err| sc_cli::Error::Service(sc_service::Error::Other(err.to_string())))?;
        for key in keys {
            println!("{}", &key);
        }
        Ok(())
    }
}

impl CliConfiguration for ListKeysCmd {
    fn shared_params(&self) -> &SharedParams {
        &self.shared_params
    }

    fn keystore_params(&self) -> Option<&KeystoreParams> {
        Some(&self.keystore_params)
    }
}

impl CliConfigurationExt for ListKeysCmd {}
