//! Machinery to populate the configuration from the CLI arguments.

use crate::configuration::Configuration;

/// An extension to the [`sc_cli::CliConfiguration`] to enable us to pass custom params.
pub trait CliConfigurationExt: SubstrateCliConfigurationProvider {
    /// Create a [`Configuration`] object from the CLI params.
    fn create_humanode_configuration<C: sc_cli::SubstrateCli>(
        &self,
        cli: &C,
        task_executor: sc_service::TaskExecutor,
    ) -> sc_cli::Result<Configuration> {
        let substrate = sc_cli::CliConfiguration::create_configuration(
            self.substrate_cli_configuration(),
            cli,
            task_executor,
        )?;
        Ok(Configuration { substrate })
    }
}

/// Indirect relation to the [`sc_cli::CliConfiguration`] for any type.
pub trait SubstrateCliConfigurationProvider {
    /// A type providing the [`sc_cli::CliConfiguration`].
    type SubstrateCliConfiguration: sc_cli::CliConfiguration;

    /// Obtain the [`sc_cli::CliConfiguration`] implementation.
    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration;
}

impl<T: sc_cli::CliConfiguration> SubstrateCliConfigurationProvider for T {
    type SubstrateCliConfiguration = T;

    fn substrate_cli_configuration(&self) -> &Self::SubstrateCliConfiguration {
        self
    }
}
