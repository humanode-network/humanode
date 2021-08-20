//! Bioauth key management subcommands.

use sc_cli::{CliConfiguration, KeystoreParams, SharedParams};
use sc_service::KeystoreContainer;
use structopt::StructOpt;

use crate::cli::CliConfigurationExt;

/// Subcommands for the `bioauth key` command.
#[derive(Debug, StructOpt)]
pub enum KeyCmd {
    /// List the bioauth keys.
    List(ListKeysCmd),
}

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
        let keys = crate::validator_key::AuraPublic::list(keystore.as_ref()).await;
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
