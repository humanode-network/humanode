//! Bioauth subcommands and related common utilities.

use structopt::StructOpt;

pub mod enroll;

/// Subcommands for the `bioauth` command.
#[derive(Debug, StructOpt)]
pub enum BioauthCmd {
    /// Run the bioauth enroll operation.
    Enroll(enroll::EnrollCmd),
}
