//! Machinery to populate the configuration from the CLI arguments.

use crate::configuration::Configuration;

/// An extension to the [`sc_cli::CliConfiguration`] to enable us to pass custom params.
pub trait CliConfigurationExt: sc_cli::CliConfiguration {
    /// Create a [`Configuration`] object from the CLI params.
    fn create_humanode_configuration<C: sc_cli::SubstrateCli>(
        &self,
        cli: &C,
        task_executor: sc_service::TaskExecutor,
    ) -> sc_cli::Result<Configuration> {
        let substrate = self.create_configuration(cli, task_executor)?;
        Ok(Configuration { substrate })
    }
}
