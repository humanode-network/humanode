//! Initializing, bootstrapping and launching the node from a provided configuration.

#![allow(clippy::type_complexity)]
use std::{marker::PhantomData, sync::Arc, time::Duration};

use humanode_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::ExecutorProvider;
use sc_consensus_aura::{ImportQueueParams, SlotDuration, SlotProportion, StartAuraParams};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_service::{Configuration, Error as ServiceError, PartialComponents, TaskManager};
use sp_consensus::SlotData;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use tracing::*;

// Native executor for the runtime based on the runtime API that is available
// at the current compile time.
native_executor_instance!(
    pub Executor,
    humanode_runtime::api::dispatch,
    humanode_runtime::native_version,
);

/// Full node client type.
type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
/// Full node backend type.
type FullBackend = sc_service::TFullBackend<Block>;
/// Full node select chain type.
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// Extract substrate partial components.
pub fn new_partial(
    config: &Configuration,
) -> Result<
    PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block, FullClient>,
        sc_transaction_pool::FullPool<Block, FullClient>,
        (
            bioauth_consensus::BioauthBlockImport<FullBackend, Block, FullClient>,
            SlotDuration,
            Duration,
        ),
    >,
    ServiceError,
> {
    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(config, None)?;
    let client = Arc::new(client);

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        Arc::clone(&client),
    );

    let select_chain = sc_consensus::LongestChain::new(Arc::clone(&backend));
    let bioauth_consensus_block_import: bioauth_consensus::BioauthBlockImport<FullBackend, _, _> =
        bioauth_consensus::BioauthBlockImport::new(Arc::clone(&client));

    let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
    let raw_slot_duration = slot_duration.slot_duration();

    let import_queue =
        sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(ImportQueueParams {
            block_import: bioauth_consensus_block_import.clone(),
            justification_import: None,
            client: Arc::clone(&client),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        raw_slot_duration,
                    );

                Ok((timestamp, slot))
            },
            spawner: &task_manager.spawn_essential_handle(),
            can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
                client.executor().clone(),
            ),
            registry: config.prometheus_registry(),
            check_for_equivocation: Default::default(),
            telemetry: None,
        })?;

    Ok(PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (
            bioauth_consensus_block_import,
            slot_duration,
            raw_slot_duration,
        ),
    })
}

/// Create a "full" node (full is in terms of substrate).
/// We don't support other node types yet either way, so this is the only way to create a node.
pub async fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (bioauth_consensus_block_import, slot_duration, raw_slot_duration),
    } = new_partial(&config)?;

    let can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;

    let proposer_factory = sc_basic_authorship::ProposerFactory::new(
        task_manager.spawn_handle(),
        Arc::clone(&client),
        Arc::clone(&transaction_pool),
        config.prometheus_registry(),
        None,
    );

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: Arc::clone(&client),
            transaction_pool: Arc::clone(&transaction_pool),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: None,
            block_announce_validator_builder: None,
            warp_sync: None,
        })?;

    let robonode_url =
        std::env::var("ROBONODE_URL").unwrap_or_else(|_| "http://127.0.0.1:3033".into());
    let robonode_client = Arc::new(robonode_client::Client {
        base_url: robonode_url,
        reqwest: reqwest::Client::new(),
    });

    let (bioauth_flow_rpc_slot, bioauth_flow_provider_slot) =
        bioauth_flow::rpc::new_liveness_data_tx_slot();

    let rpc_extensions_builder = {
        let client = Arc::clone(&client);
        let pool = Arc::clone(&transaction_pool);
        let robonode_client = Arc::clone(&robonode_client);
        let bioauth_flow_rpc_slot = Arc::new(bioauth_flow_rpc_slot);
        Box::new(move |deny_unsafe, _| {
            Ok(humanode_rpc::create(humanode_rpc::Deps {
                client: Arc::clone(&client),
                pool: Arc::clone(&pool),
                deny_unsafe,
                robonode_client: Arc::clone(&robonode_client),
                bioauth_flow_slot: Arc::clone(&bioauth_flow_rpc_slot),
                bioauth_runtime_handle: tokio::runtime::Handle::current(),
            }))
        })
    };

    let rpc_port = config.rpc_http.expect("HTTP RPC must be on").port();
    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: Arc::clone(&network),
        client: Arc::clone(&client),
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool: Arc::clone(&transaction_pool),
        rpc_extensions_builder,
        on_demand: None,
        remote_blockchain: None,
        backend,
        system_rpc_tx,
        config,
        telemetry: None,
    })?;

    let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _, _>(
        StartAuraParams {
            slot_duration,
            client: Arc::clone(&client),
            select_chain,
            block_import: bioauth_consensus_block_import,
            proposer_factory,
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        raw_slot_duration,
                    );

                Ok((timestamp, slot))
            },
            force_authoring,
            backoff_authoring_blocks,
            keystore: keystore_container.sync_keystore(),
            can_author_with,
            sync_oracle: Arc::clone(&network),
            justification_sync_link: network,
            block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
            max_block_proposal_slot_portion: None,
            telemetry: None,
        },
    )?;

    // The AURA authoring task is considered essential, i.e. if it
    // fails we take down the service with it.
    task_manager
        .spawn_essential_handle()
        .spawn_blocking("aura", aura);

    network_starter.start_network();

    let mut flow = bioauth_flow::flow::Flow {
        liveness_data_provider: bioauth_flow::rpc::Provider::new(bioauth_flow_provider_slot),
        robonode_client,
        validator_public_key_type: PhantomData,
        validator_signer_type: PhantomData,
    };

    let webapp_url = std::env::var("WEBAPP_URL")
        .unwrap_or_else(|_| "https://webapp-test-1.dev.humanode.io".into());
    // TODO: more advanced host address detection is needed to things work within the same LAN.
    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| format!("http://localhost:{}", rpc_port));
    let webapp_qrcode =
        crate::qrcode::WebApp::new(&webapp_url, &rpc_url).map_err(ServiceError::Other)?;

    let bioauth_flow_future = {
        let client = Arc::clone(&client);
        let keystore = keystore_container.keystore();
        let transaction_pool = Arc::clone(&transaction_pool);
        Box::pin(async move {
            info!("bioauth flow starting up");

            let aura_public_key =
                crate::validator_key::AuraPublic::from_keystore(keystore.as_ref())
                    .await
                    .expect("vector has to be of length 1 at this point");

            let should_enroll = std::env::var("ENROLL").unwrap_or_default() == "true";
            if should_enroll {
                info!("bioauth flow - enrolling in progress");

                webapp_qrcode.print();

                flow.enroll(&aura_public_key).await.expect("enroll failed");

                info!("bioauth flow - enrolling complete");
            }

            info!("bioauth flow - authentication in progress");

            webapp_qrcode.print();

            let aura_signer = crate::validator_key::AuraSigner {
                keystore: Arc::clone(&keystore),
                public_key: aura_public_key,
            };

            let authenticate_response = loop {
                let result = flow.authenticate(&aura_signer).await;
                match result {
                    Ok(v) => break v,
                    Err(error) => {
                        error!(message = "bioauth flow - authentication failure", ?error);
                    }
                };
            };

            info!("bioauth flow - authentication complete");

            info!(message = "We've obtained an auth ticket", auth_ticket = ?authenticate_response.auth_ticket);

            let authenticate = pallet_bioauth::Authenticate {
                ticket: authenticate_response.auth_ticket.into(),
                ticket_signature: authenticate_response.auth_ticket_signature.into(),
            };
            let call = pallet_bioauth::Call::authenticate(authenticate);

            let ext = humanode_runtime::UncheckedExtrinsic::new_unsigned(call.into());

            let at = client.chain_info().best_hash;
            transaction_pool
                .pool()
                .submit_and_watch(
                    &sp_runtime::generic::BlockId::Hash(at),
                    sp_runtime::transaction_validity::TransactionSource::Local,
                    ext.into(),
                )
                .await
                .unwrap();
        })
    };

    task_manager
        .spawn_handle()
        .spawn_blocking("bioauth-flow", bioauth_flow_future);

    Ok(task_manager)
}
