//! Bioauth key management subcommands.

use structopt::StructOpt;

pub mod insert;
pub mod list;

/// Subcommands for the `bioauth key` command.
#[derive(Debug, StructOpt)]
pub enum KeyCmd {
    /// Insert the bioauth key.
    Insert(insert::InsertKeyCmd),
    /// List the bioauth keys.
    List(list::ListKeysCmd),
}
