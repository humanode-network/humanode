//! Bioauth key management subcommands.

use sc_cli::{utils, CliConfiguration, KeystoreParams, SharedParams};
use sc_service::KeystoreContainer;
use sp_application_crypto::AppKey;
use sp_core::Pair;
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
    /// The bioauth key is already inserted error.
    #[error("The bioauth key is already inserted")]
    AlreadyInserted,
}

impl InsertKeyCmd {
    /// Run the list command.
    pub async fn run(&self, keystore_container: KeystoreContainer) -> sc_cli::Result<()> {
        let keystore = keystore_container.keystore();

        // We should verify that there is no bioauth key at the keystore.
        let mut current_keys = crate::validator_key::AppCryptoPublic::<
            sp_consensus_babe::AuthorityId,
        >::list(keystore.as_ref())
        .await
        .map_err(|err| sc_cli::Error::Service(sc_service::Error::Other(err.to_string())))?;

        if current_keys.next().is_some() {
            return Err(sc_cli::Error::Service(sc_service::Error::Other(
                InsertKeyError::AlreadyInserted.to_string(),
            )));
        }

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
