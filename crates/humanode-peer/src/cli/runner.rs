//! [`Runner`] utility.

use std::marker::PhantomData;

use futures::Future;
use sc_cli::{Error as CliError, Result, SubstrateCli};
use sc_service::{Error as ServiceError, TaskManager};

use crate::configuration::Configuration;

use super::{CliConfigurationExt, Root};

/// Run a future until it completes or a signal is recevied.
async fn with_signal<F, E>(future: F) -> std::result::Result<(), E>
where
    F: Future<Output = std::result::Result<(), E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    tokio::select! {
        res = future => res?,
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Got Ctrl+C");
        }
    };
    Ok(())
}

/// A runner utility encapsulates the logic of handling the execution flow (i.e. signal handling and
/// task manager clean shutdown) for the regular node runtime and the ad-hoc commands.
pub struct Runner<C: SubstrateCli> {
    /// Configuration.
    config: Configuration,
    /// The type of the cli implementation.
    cli_type: PhantomData<C>,
}

impl<C: SubstrateCli> Runner<C> {
    /// Create a new runner for the specified command.
    pub fn new<T: CliConfigurationExt>(cli: &Root, command: &T) -> Result<Self> {
        let runtime_handle = tokio::runtime::Handle::current();
        let config = command.create_humanode_configuration(cli, runtime_handle)?;

        Ok(Self {
            config,
            cli_type: PhantomData,
        })
    }

    /// Run the task manager to completion, or till the signal.
    pub async fn run_node<F, E>(
        self,
        initialize: impl FnOnce(Configuration) -> F,
    ) -> std::result::Result<(), E>
    where
        F: Future<Output = std::result::Result<TaskManager, E>>,
        E: std::error::Error + Send + Sync + 'static + From<ServiceError>,
    {
        let mut task_manager = initialize(self.config).await?;
        let future = task_manager.future();
        let future = with_signal(future);
        let res = future.await;
        Ok(res?)
    }

    /// Run some tasks with task manager.
    /// The runner is executing till completion, or until till the signal is received.
    pub async fn run_tasks<R, F, E>(
        self,
        runner: impl FnOnce(Configuration) -> R,
    ) -> std::result::Result<(), E>
    where
        R: Future<Output = std::result::Result<(F, TaskManager), E>>,
        F: Future<Output = std::result::Result<(), E>>,
        E: std::error::Error + Send + Sync + 'static + From<ServiceError> + From<CliError>,
    {
        let (future, task_manager) = runner(self.config).await?;
        let future = with_signal(future);
        drop(task_manager);
        future.await
    }

    /// Execute asynchronously.
    pub async fn async_run<F, E>(
        self,
        runner: impl FnOnce(Configuration) -> F,
    ) -> std::result::Result<(), E>
    where
        F: Future<Output = std::result::Result<(), E>>,
        E: std::error::Error + Send + Sync + 'static + From<ServiceError> + From<CliError>,
    {
        let future = runner(self.config);
        let future = with_signal(future);
        future.await
    }

    /// Execute syncronously.
    pub fn sync_run<E>(
        self,
        runner: impl FnOnce(Configuration) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), E>
    where
        E: std::error::Error + Send + Sync + 'static + From<ServiceError>,
    {
        runner(self.config)
    }

    /// Get an immutable reference to the node [`Configuration`].
    pub fn config(&self) -> &Configuration {
        &self.config
    }
}
