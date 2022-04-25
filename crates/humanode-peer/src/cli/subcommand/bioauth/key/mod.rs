//! Bioauth key management subcommands.

use structopt::StructOpt;

pub mod generate;
pub mod insert;
pub mod inspect;
pub mod list;

/// Keystore bioauth identifier used at the keystore.
pub type KeystoreBioauthId = keystore_bioauth_account_id::KeystoreBioauthAccountId;
/// Keystore bioauth key pair scheme type used at the keystore.
pub type KeystoreBioauthPair = <<KeystoreBioauthId as sp_application_crypto::CryptoType>::Pair as sp_application_crypto::AppPair>::Generic;

/// Subcommands for the `bioauth key` command.
#[derive(Debug, StructOpt)]
pub enum KeyCmd {
    /// Generate the bioauth key.
    Generate(generate::GenerateKeyCmd),
    /// Inspect the bioauth key.
    Inspect(inspect::InspectKeyCmd),
    /// Insert the bioauth key.
    Insert(insert::InsertKeyCmd),
    /// List the bioauth keys.
    List(list::ListKeysCmd),
}
