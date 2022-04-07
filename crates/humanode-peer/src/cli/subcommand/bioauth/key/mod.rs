//! Bioauth key management subcommands.

use structopt::StructOpt;

pub mod generate;
pub mod insert;
pub mod inspect;
pub mod list;

/// Bioauth identifier used at the keystore.
pub type BioauthId = humanode_runtime::BioauthId;
/// Bioauth key pair scheme type used at the keystore.
pub type BioauthPair = sp_core::sr25519::Pair;

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
