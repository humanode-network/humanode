//! Command line arguments processing.

use crate::{
    chain_spec,
    cli::{Cli, Subcommand},
    service,
};

use humanode_runtime::Block;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use sc_service::PartialComponents;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Humanode Node".into()
    }

    fn impl_version() -> String {
        "0".to_owned()
    }

    fn description() -> String {
        "Biologically verified human-nodes as a basis for a fair financial system.".into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/humanode-network/humanode/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2021
    }

    fn load_spec(&self, _id: &str) -> std::result::Result<Box<dyn ChainSpec>, String> {
        Ok(Box::new(chain_spec::local_testnet_config()?))
    }

    fn native_runtime_version(_chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &humanode_runtime::VERSION
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, backend), task_manager))
            })
        }
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;

                runner.sync_run(|config| cmd.run::<Block, service::Executor>(config))
            } else {
                Err(
                    "Benchmarking wasn't enabled when building the node. You can enable it with \
				     `--features runtime-benchmarks`."
                        .into(),
                )
            }
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config)
                    .await
                    .map_err(sc_cli::Error::Service)
            })
        }
    }
}
