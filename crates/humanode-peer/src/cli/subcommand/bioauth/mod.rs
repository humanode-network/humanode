//! Bioauth subcommands and related common utilities.

use structopt::StructOpt;

pub mod api_versions;
pub mod authurl;
pub mod key;

/// Subcommands for the `bioauth` command.
#[derive(Debug, StructOpt)]
pub enum BioauthCmd {
    /// Bioauth key utilities.
    Key(key::KeyCmd),
    /// Web App URL with bound RPC URL.
    AuthUrl(authurl::AuthUrlCmd),
    /// API versions print.
    ApiVersions(api_versions::ApiVersionsCmd),
}
