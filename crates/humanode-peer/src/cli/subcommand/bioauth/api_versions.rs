//! ApiVersions subcommand logic.

use sc_cli::{CliConfiguration, SharedParams};
use serde_json::json;
use structopt::StructOpt;

use crate::{cli::CliConfigurationExt, version};

/// The `bioauth api-verisons` command.
#[derive(Debug, StructOpt)]
pub struct ApiVersionsCmd {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[structopt(flatten)]
    pub shared_params: SharedParams,
}

impl ApiVersionsCmd {
    /// Run the api-versions command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let current_api_versions = version::API_VERSIONS;
        let json = json!(current_api_versions);
        println!(
            "{}",
            serde_json::to_string_pretty(&json).expect("Json pretty print failed")
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
