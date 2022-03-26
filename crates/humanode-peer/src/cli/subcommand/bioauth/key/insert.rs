//! Bioauth key management subcommands.

use sc_cli::{utils, CliConfiguration, KeystoreParams, SharedParams};
use sc_service::KeystoreContainer;
use sp_application_crypto::{AppKey, AppPublic};
use sp_core::Pair;
use sp_keystore::CryptoStore;
use std::sync::Arc;
use structopt::StructOpt;

use crate::cli::CliConfigurationExt;

/// The `bioauth key insert` command.
#[derive(Debug, StructOpt)]
pub struct InsertKeyCmd {
    /// The secret key uri (mnemonic).
    #[structopt(long)]
    pub suri: String,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub shared_params: SharedParams,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub keystore_params: KeystoreParams,
}

/// An error that occured during key insert.
#[derive(thiserror::Error, Debug)]
pub enum InsertKeyError {
    /// Something went wrong while extracting the list of bioauth keys.
    #[error("Unable to extract keys list from keystore: {0}")]
    UnableToExtractKeys(sp_keystore::Error),
    /// The bioauth key is already inserted error.
    #[error("The bioauth key is already inserted")]
    AlreadyInserted,
}

/// A helper function to verify that there is no bioauth key at the keystore yet.
pub async fn does_bioauth_key_already_exist<PK: AppPublic>(
    crypto_store: Arc<dyn CryptoStore>,
) -> Result<(), InsertKeyError> {
    let mut current_keys = crate::validator_key::AppCryptoPublic::<PK>::list(crypto_store.as_ref())
        .await
        .map_err(InsertKeyError::UnableToExtractKeys)?;

    if current_keys.next().is_some() {
        return Err(InsertKeyError::AlreadyInserted);
    }
    Ok(())
}

impl InsertKeyCmd {
    /// Run the list command.
    pub async fn run(&self, keystore_container: KeystoreContainer) -> sc_cli::Result<()> {
        let keystore = keystore_container.keystore();

        does_bioauth_key_already_exist::<sp_consensus_babe::AuthorityId>(Arc::clone(&keystore))
            .await
            .map_err(|err| sc_cli::Error::Service(sc_service::Error::Other(err.to_string())))?;

        let pair = utils::pair_from_suri::<sp_core::sr25519::Pair>(self.suri.as_ref(), None)?;
        let public = pair.public().to_vec();
        keystore
            .insert_unknown(
                sp_consensus_babe::AuthorityId::ID,
                self.suri.as_ref(),
                &public[..],
            )
            .await
            .map_err(|_| sc_cli::Error::KeyStoreOperation)?;
        Ok(())
    }
}

impl CliConfiguration for InsertKeyCmd {
    fn shared_params(&self) -> &SharedParams {
        &self.shared_params
    }

    fn keystore_params(&self) -> Option<&KeystoreParams> {
        Some(&self.keystore_params)
    }
}

impl CliConfigurationExt for InsertKeyCmd {}
