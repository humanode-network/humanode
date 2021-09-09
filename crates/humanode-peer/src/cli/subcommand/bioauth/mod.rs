//! Bioauth subcommands and related common utilities.

use structopt::StructOpt;

pub mod key;

/// Subcommands for the `bioauth` command.
#[derive(Debug, StructOpt)]
pub enum BioauthCmd {
    /// Bioauth key utilities.
    Key(key::KeyCmd),
}
