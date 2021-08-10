//! [`Runner`] utility.

use futures::{future::FutureExt, Future};
use sc_cli::{CliConfiguration, Error as CliError, Result, SubstrateCli};
use sc_service::{Configuration, Error as ServiceError, TaskManager, TaskType};
use std::marker::PhantomData;

/// Run the given future and then clean shutdown the task manager before returning the control.
async fn with_clean_shutdown<F, O>(fut: F, task_manager: TaskManager) -> O
where
    F: Future<Output = O>,
{
    let res = fut.await;
    task_manager.clean_shutdown().await;
    res
}

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
    pub fn new<T: CliConfiguration>(cli: &C, command: &T) -> Result<Runner<C>> {
        let runtime_handle = tokio::runtime::Handle::current();

        let task_executor = move |fut, task_type| match task_type {
            TaskType::Async => runtime_handle.spawn(fut).map(drop),
            TaskType::Blocking => runtime_handle
                .spawn_blocking(move || futures::executor::block_on(fut))
                .map(drop),
        };

        Ok(Runner {
            config: command.create_configuration(cli, task_executor.into())?,
            cli_type: PhantomData,
        })
    }

    /// Run the task manager to completion, or till the signal, with clean shutdown.
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
        task_manager.clean_shutdown().await;
        Ok(res?)
    }

    /// Execute asyncronously.
    /// The runner is executing till completion, or until till the signal is received.
    /// Task manager is shutdown cleanly at the end (even on error).
    pub async fn async_run<R, F, E>(
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
        let future = with_clean_shutdown(future, task_manager);
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

    /// Get an immutable reference to the node Configuration
    pub fn config(&self) -> &Configuration {
        &self.config
    }
}

/// Log information about the node itself.
pub fn print_node_infos<C: SubstrateCli>(config: &Configuration) {
    use tracing::info;

    info!("{}", C::impl_name());
    info!("✌️  version {}", C::impl_version());
    info!("📋 Chain specification: {}", config.chain_spec.name());
    info!("🏷 Node name: {}", config.network.node_name);
    info!("👤 Role: {}", config.display_role());
    info!(
        "💾 Database: {} at {}",
        config.database,
        config
            .database
            .path()
            .map_or_else(|| "<unknown>".to_owned(), |p| p.display().to_string())
    );
    info!(
        "⛓  Native runtime: {}",
        C::native_runtime_version(&config.chain_spec)
    );
}
