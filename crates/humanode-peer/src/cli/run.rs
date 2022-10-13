//! The main entrypoint.

use std::sync::Arc;

use frame_benchmarking_cli::*;
use humanode_runtime::Block;
use sc_service::PartialComponents;
#[cfg(feature = "runtime-benchmarks")]
use sp_keyring::Sr25519Keyring;

use super::{bioauth, Root, Subcommand};
#[cfg(feature = "runtime-benchmarks")]
use crate::benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder};
use crate::service;

/// Parse command line arguments and run the requested operation.
pub async fn run() -> sc_cli::Result<()> {
    let root: Root = sc_cli::SubstrateCli::from_args();

    match &root.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&root),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.substrate.chain_spec, config.substrate.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .run_tasks(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        import_queue,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, import_queue), task_manager))
                })
                .await
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .run_tasks(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, config.substrate.database), task_manager))
                })
                .await
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .run_tasks(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, config.substrate.chain_spec), task_manager))
                })
                .await
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .run_tasks(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        import_queue,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, import_queue), task_manager))
                })
                .await
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.substrate.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .run_tasks(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        backend,
                        ..
                    } = service::new_partial(&config)?;
                    let aux_revert = Box::new(|client, backend, blocks| {
                        sc_consensus_babe::revert(Arc::clone(&client), backend, blocks)?;
                        sc_finality_grandpa::revert(client, blocks)?;
                        Ok(())
                    });
                    Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
                })
                .await
        }
        Some(Subcommand::Bioauth(bioauth::BioauthCmd::Key(bioauth::key::KeyCmd::Generate(
            cmd,
        )))) => cmd.run().await,
        Some(Subcommand::Bioauth(bioauth::BioauthCmd::Key(bioauth::key::KeyCmd::Inspect(cmd)))) => {
            cmd.run().await
        }
        Some(Subcommand::Bioauth(bioauth::BioauthCmd::Key(bioauth::key::KeyCmd::Insert(cmd)))) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .run_tasks(|config| async move {
                    let (keystore_container, task_manager) = service::keystore_container(&config)?;
                    Ok((cmd.run(keystore_container), task_manager))
                })
                .await
        }
        Some(Subcommand::Bioauth(bioauth::BioauthCmd::Key(bioauth::key::KeyCmd::List(cmd)))) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .run_tasks(|config| async move {
                    let (keystore_container, task_manager) = service::keystore_container(&config)?;
                    Ok((cmd.run(keystore_container), task_manager))
                })
                .await
        }
        Some(Subcommand::Bioauth(bioauth::BioauthCmd::ApiVersions(cmd))) => cmd.run().await,
        Some(Subcommand::Bioauth(bioauth::BioauthCmd::AuthUrl(cmd))) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .async_run(|config| async move { cmd.run(config.bioauth_flow).await })
                .await
        }
        Some(Subcommand::Ethereum(cmd)) => cmd.run().await,
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;

            runner.sync_run(|config| {
                // This switch needs to be in the client, since the client decides
                // which sub-commands it wants to support.
                match cmd {
                    BenchmarkCmd::Pallet(cmd) => {
                        if !cfg!(feature = "runtime-benchmarks") {
                            return Err(
                                "Runtime benchmarking wasn't enabled when building the node. \
							You can enable it with `--features runtime-benchmarks`."
                                    .into(),
                            );
                        }

                        cmd.run::<Block, service::ExecutorDispatch>(config.substrate)
                    }
                    BenchmarkCmd::Block(cmd) => {
                        let partial = service::new_partial(&config)?;
                        cmd.run(partial.client)
                    }
                    #[cfg(not(feature = "runtime-benchmarks"))]
                    BenchmarkCmd::Storage(_) => Err(
                        "Storage benchmarking can be enabled with `--features runtime-benchmarks`."
                            .into(),
                    ),
                    #[cfg(feature = "runtime-benchmarks")]
                    BenchmarkCmd::Storage(cmd) => {
                        let partial = service::new_partial(&config)?;
                        let db = partial.backend.expose_db();
                        let storage = partial.backend.expose_storage();

                        cmd.run(config.substrate, partial.client, db, storage)
                    }
                    #[cfg(not(feature = "runtime-benchmarks"))]
                    BenchmarkCmd::Overhead(_) => Err(
                        "Overhead benchmarking can be enabled with `--features runtime-benchmarks.`".into()
                    ),
                    #[cfg(feature = "runtime-benchmarks")]
                    BenchmarkCmd::Overhead(cmd) => {
                        let partial = service::new_partial(&config)?;
                        let ext_builder = RemarkBuilder::new(Arc::clone(&partial.client));
                        cmd.run(
                            config.substrate,
                            partial.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_builder,
                        )
                    },
                    #[cfg(not(feature = "runtime-benchmarks"))]
                    BenchmarkCmd::Extrinsic(_) => Err(
                        "Extrinsic benchmarking can be enabled with `--features runtime-benchmarks.`".into()
                    ),
                    #[cfg(feature = "runtime-benchmarks")]
                    BenchmarkCmd::Extrinsic(cmd) => {
                        let partial = service::new_partial(&config)?;
                        let ext_factory = ExtrinsicFactory(vec![
                            Box::new(RemarkBuilder::new(Arc::clone(&partial.client))),
                            Box::new(TransferKeepAliveBuilder::new(
                                Arc::clone(&partial.client),
                                Sr25519Keyring::Bob.to_account_id(),
                                500,
                            )),
                        ]);
                        cmd.run(
                            partial.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_factory,
                        )
                    }
                    BenchmarkCmd::Machine(cmd) => {
                        cmd.run(&config.substrate, SUBSTRATE_REFERENCE_HARDWARE.clone())
                    }
                }
            })
        }
        Some(Subcommand::FrontierDb(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner.sync_run(|config| {
                let partial = service::new_partial(&config)?;
                let frontier_backend = partial.other.4;
                cmd.run(partial.client, frontier_backend)
            })
        }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .run_tasks(|config| async move {
                    // we don't need any of the components of new_partial, just a runtime, or a task
                    // manager to do `async_run`.
                    let registry = config
                        .substrate
                        .prometheus_config
                        .as_ref()
                        .map(|cfg| &cfg.registry);
                    let task_manager = sc_service::TaskManager::new(
                        config.substrate.tokio_handle.clone(),
                        registry,
                    )
                    .map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
                    Ok((
                        cmd.run::<Block, service::ExecutorDispatch>(config.substrate),
                        task_manager,
                    ))
                })
                .await
        }
        #[cfg(not(feature = "try-runtime"))]
        Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
				You can enable it with `--features try-runtime`."
            .into()),
        None => {
            let runner = root.create_humanode_runner(&root.run)?;
            sc_cli::print_node_infos::<Root>(&runner.config().substrate);
            print_build_info();
            runner
                .run_node(|config| async move {
                    service::new_full(config)
                        .await
                        .map_err(sc_cli::Error::Service)
                })
                .await
        }
    }
}

/// Print the extended version information.
fn print_build_info() {
    tracing::info!("   Build info - commit sha: {}", crate::build_info::GIT_SHA);
    tracing::info!(
        "   Build info - cargo profile: {}",
        crate::build_info::CARGO_PROFILE
    );
    tracing::info!(
        "   Build info - cargo features: {}",
        crate::build_info::CARGO_FEATURES
    );
}
