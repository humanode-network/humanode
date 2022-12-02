//! `ApiVersions` subcommand logic.

use sc_cli::{CliConfiguration, SharedParams};
use serde_json::json;

use crate::{api_versions, cli::CliConfigurationExt};

/// The `bioauth api-versions` command.
#[derive(Debug, clap::Parser)]
pub struct ApiVersionsCmd {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[command(flatten)]
    pub shared_params: SharedParams,
}

impl ApiVersionsCmd {
    /// Run the api-versions command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let json = json!(api_versions::API_VERSIONS);
        println!(
            "{}",
            serde_json::to_string_pretty(&json).expect("JSON pretty print failed")
        );
        Ok(())
    }
}

impl CliConfiguration for ApiVersionsCmd {
    fn shared_params(&self) -> &SharedParams {
        &self.shared_params
    }
}

impl CliConfigurationExt for ApiVersionsCmd {}
