use sc_cli::{CliConfiguration, Error as CliError, Result, SubstrateCli};

use chrono::prelude::*;
use futures::{future, future::FutureExt, pin_mut, select, Future};
use log::info;
use sc_service::{Configuration, Error as ServiceError, TaskManager, TaskType};
// use sp_utils::metrics::{TOKIO_THREADS_ALIVE, TOKIO_THREADS_TOTAL};
use std::marker::PhantomData;

#[cfg(target_family = "unix")]
async fn main<F, E>(func: F) -> std::result::Result<(), E>
where
    F: Future<Output = std::result::Result<(), E>> + future::FusedFuture,
    E: std::error::Error + Send + Sync + 'static + From<ServiceError>,
{
    use tokio::signal::unix::{signal, SignalKind};

    let mut stream_int = signal(SignalKind::interrupt()).map_err(ServiceError::Io)?;
    let mut stream_term = signal(SignalKind::terminate()).map_err(ServiceError::Io)?;

    let t1 = stream_int.recv().fuse();
    let t2 = stream_term.recv().fuse();
    let t3 = func;

    pin_mut!(t1, t2, t3);

    select! {
        _ = t1 => {},
        _ = t2 => {},
        res = t3 => res?,
    }

    Ok(())
}

#[cfg(not(unix))]
async fn main<F, E>(func: F) -> std::result::Result<(), E>
where
    F: Future<Output = std::result::Result<(), E>> + future::FusedFuture,
    E: std::error::Error + Send + Sync + 'static + From<ServiceError>,
{
    use tokio::signal::ctrl_c;

    let t1 = ctrl_c().fuse();
    let t2 = func;

    pin_mut!(t1, t2);

    select! {
        _ = t1 => {},
        res = t2 => res?,
    }

    Ok(())
}

/// Run an humanode node until get exit signal
async fn run_until_exit<F, E>(future: F, task_manager: TaskManager) -> std::result::Result<(), E>
where
    F: Future<Output = std::result::Result<(), E>> + future::Future,
    E: std::error::Error + Send + Sync + 'static + From<ServiceError>,
{
    let f = future.fuse();
    pin_mut!(f);

    main(f).await?;
    task_manager.clean_shutdown().await;

    Ok(())
}

/// A Substrate CLI runtime that can be used to run a node or a command
pub struct Runner<C: SubstrateCli> {
    /// Configuration data
    config: Configuration,
    /// Phantom Data
    phantom: PhantomData<C>,
}

impl<C: SubstrateCli> Runner<C> {
    /// Create a new runtime with the command provided in argument
    pub fn new<T: CliConfiguration>(cli: &C, command: &T) -> Result<Runner<C>> {
        let runtime_handle = tokio::runtime::Handle::current();
        let runtime_handle_clone = runtime_handle.clone();

        let task_executor = move |fut, task_type| match task_type {
            TaskType::Async => runtime_handle_clone.spawn(fut).map(drop),
            TaskType::Blocking => runtime_handle_clone
                .spawn_blocking(move || futures::executor::block_on(fut))
                .map(drop),
        };

        Ok(Runner {
            config: command.create_configuration(cli, task_executor.into())?,
            phantom: PhantomData,
        })
    }

    /// Log information about the node itself.
    ///
    /// # Example:
    ///
    /// ```text
    /// 2020-06-03 16:14:21 Substrate Node
    /// 2020-06-03 16:14:21 ✌️  version 2.0.0-rc3-f4940588c-x86_64-linux-gnu
    /// 2020-06-03 16:14:21 ❤️  by Parity Technologies <admin@parity.io>, 2017-2020
    /// 2020-06-03 16:14:21 📋 Chain specification: Flaming Fir
    /// 2020-06-03 16:14:21 🏷 Node name: jolly-rod-7462
    /// 2020-06-03 16:14:21 👤 Role: FULL
    /// 2020-06-03 16:14:21 💾 Database: RocksDb at /tmp/c/chains/flamingfir7/db
    /// 2020-06-03 16:14:21 ⛓  Native runtime: node-251 (substrate-node-1.tx1.au10)
    /// ```
    fn print_node_infos(&self) {
        print_node_infos::<C>(self.config())
    }

    /// A helper function that runs a node with tokio and stops if the process receives the signal
    /// `SIGTERM` or `SIGINT`.
    pub async fn run_node_until_exit<F, E>(
        self,
        initialize: impl FnOnce(Configuration) -> F,
    ) -> std::result::Result<(), E>
    where
        F: Future<Output = std::result::Result<TaskManager, E>>,
        E: std::error::Error + Send + Sync + 'static + From<ServiceError>,
    {
        self.print_node_infos();
        let mut task_manager = initialize(self.config).await?;
        let res = main(task_manager.future().fuse()).await;
        task_manager.clean_shutdown().await;
        Ok(res?)
    }

    /// A helper function that runs a command with the configuration of this node.
    pub fn sync_run<E>(
        self,
        runner: impl FnOnce(Configuration) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), E>
    where
        E: std::error::Error + Send + Sync + 'static + From<ServiceError>,
    {
        runner(self.config)
    }

    /// A helper function that runs a future with tokio and stops if the process receives
    /// the signal `SIGTERM` or `SIGINT`.
    pub async fn async_run<F, E>(
        self,
        runner: impl FnOnce(Configuration) -> std::result::Result<(F, TaskManager), E>,
    ) -> std::result::Result<(), E>
    where
        F: Future<Output = std::result::Result<(), E>>,
        E: std::error::Error + Send + Sync + 'static + From<ServiceError> + From<CliError>,
    {
        let (future, task_manager) = runner(self.config)?;
        run_until_exit::<_, E>(future, task_manager).await
    }

    /// Get an immutable reference to the node Configuration
    pub fn config(&self) -> &Configuration {
        &self.config
    }
}

/// Log information about the node itself.
pub fn print_node_infos<C: SubstrateCli>(config: &Configuration) {
    info!("{}", C::impl_name());
    info!("✌️  version {}", C::impl_version());
    info!(
        "❤️  by {}, {}-{}",
        C::author(),
        C::copyright_start_year(),
        Local::today().year()
    );
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
