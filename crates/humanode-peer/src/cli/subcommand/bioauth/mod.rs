//! Bioauth subcommands and related common utilities.

pub mod api_versions;
pub mod authurl;
pub mod key;

/// Subcommands for the `bioauth` command.
#[derive(Debug, clap::Subcommand)]
pub enum BioauthCmd {
    /// Bioauth key utilities.
    #[command(subcommand)]
    Key(key::KeyCmd),
    /// Web App URL with bound RPC URL.
    AuthUrl(authurl::AuthUrlCmd),
    /// API versions print.
    ApiVersions(api_versions::ApiVersionsCmd),
}
