//! Build the import queue depending on either (sealing) or (babe, grandpa) cases.

use std::sync::Arc;

use fc_consensus::FrontierBlockImport;
use sc_consensus_babe::BabeLink;
use sc_service::{Error as ServiceError, TaskManager};
use sc_telemetry::TelemetryHandle;

use super::{
    inherents, Block, FrontierBackend, FullBabe, FullBoxBlockImport, FullClient, FullGrandpa,
    FullSelectChain,
};

/// Build the import queue for sealing case.
pub fn sealing(
    client: Arc<FullClient>,
    config: &sc_service::Configuration,
    task_manager: &TaskManager,
    frontier_backend: Arc<FrontierBackend>,
) -> (
    sc_consensus::DefaultImportQueue<Block, FullClient>,
    FullBoxBlockImport,
) {
    let block_import = FrontierBlockImport::new(
        Arc::clone(&client),
        Arc::clone(&client),
        Arc::clone(&frontier_backend),
    );

    let import_queue = sc_consensus_manual_seal::import_queue(
        Box::new(block_import.clone()),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    );

    (import_queue, Box::new(block_import))
}

/// Build the import queue for babe and grandpa case.
///
/// Allow all arguments to be passed together for more clarity.
#[allow(clippy::too_many_arguments)]
pub fn babe_grandpa(
    client: Arc<FullClient>,
    config: &sc_service::Configuration,
    task_manager: &TaskManager,
    select_chain: FullSelectChain,
    babe_block_import: FullBabe,
    babe_link: BabeLink<Block>,
    grandpa_block_import: FullGrandpa,
    frontier_backend: Arc<FrontierBackend>,
    inherent_data_providers_creator: inherents::Creator<FullClient>,
    telemetry: Option<TelemetryHandle>,
) -> Result<
    (
        sc_consensus::DefaultImportQueue<Block, FullClient>,
        FullBoxBlockImport,
    ),
    ServiceError,
> {
    let block_import = FrontierBlockImport::new(
        babe_block_import,
        Arc::clone(&client),
        Arc::clone(&frontier_backend),
    );

    let import_queue = sc_consensus_babe::import_queue(
        babe_link.clone(),
        block_import.clone(),
        Some(Box::new(grandpa_block_import)),
        Arc::clone(&client),
        select_chain.clone(),
        inherents::ForImport(inherent_data_providers_creator),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        telemetry,
    )?;

    Ok((import_queue, Box::new(block_import)))
}
