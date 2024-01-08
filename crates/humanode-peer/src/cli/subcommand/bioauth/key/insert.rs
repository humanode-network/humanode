//! Bioauth key insert subcommand logic.

use std::sync::Arc;

use sc_cli::{utils, CliConfiguration, KeystoreParams, SharedParams};
use sc_service::KeystoreContainer;
use sp_application_crypto::{AppCrypto, AppPublic};
use sp_core::Pair;
use sp_keystore::Keystore;

use super::KeystoreBioauthId;
use crate::cli::CliConfigurationExt;

/// The `bioauth key insert` command.
#[derive(Debug, clap::Parser)]
pub struct InsertKeyCmd {
    /// The secret key uri (mnemonic).
    #[arg(long, short = 'm')]
    pub suri: String,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[command(flatten)]
    pub shared_params: SharedParams,

    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[command(flatten)]
    pub keystore_params: KeystoreParams,
}

/// An error that occurred during key insert.
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
pub async fn ensure_bioauth_key_absent<PK: AppPublic>(
    keystore: Arc<dyn Keystore>,
) -> Result<(), InsertKeyError> {
    let mut current_keys = crate::validator_key::AppCryptoPublic::<PK>::list(keystore.as_ref())
        .await
        .map_err(InsertKeyError::UnableToExtractKeys)?;

    if current_keys.next().is_some() {
        return Err(InsertKeyError::AlreadyInserted);
    }
    Ok(())
}

/// A helper function to insert bioauth key into the keystore.
pub async fn insert_bioauth_key<PK: AppPublic>(
    suri: &str,
    keystore: Arc<dyn Keystore>,
) -> sc_cli::Result<()> {
    // We don't use a password for keystore at the current moment. That's why None is passed.
    let pair = utils::pair_from_suri::<<PK as AppCrypto>::Pair>(suri, None)?;
    let public = pair.public().as_ref().to_vec();
    keystore
        .insert(PK::ID, suri, &public[..])
        .map_err(|_| sc_cli::Error::KeystoreOperation)?;
    Ok(())
}

impl InsertKeyCmd {
    /// Run the insert command.
    pub async fn run(&self, keystore_container: KeystoreContainer) -> sc_cli::Result<()> {
        let keystore = keystore_container.keystore();

        ensure_bioauth_key_absent::<KeystoreBioauthId>(Arc::clone(&keystore))
            .await
            .map_err(|err| sc_cli::Error::Service(sc_service::Error::Other(err.to_string())))?;

        insert_bioauth_key::<KeystoreBioauthId>(self.suri.as_ref(), keystore).await?;
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
