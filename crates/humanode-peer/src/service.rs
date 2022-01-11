//! Initializing, bootstrapping and launching the node from a provided configuration.

#![allow(clippy::type_complexity)]
use std::{sync::Arc, time::Duration};

use humanode_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::ExecutorProvider;
use sc_consensus_aura::{ImportQueueParams, SlotDuration, SlotProportion, StartAuraParams};
pub use sc_executor::NativeElseWasmExecutor;
use sc_finality_grandpa::SharedVoterState;
use sc_service::{Error as ServiceError, KeystoreContainer, PartialComponents, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus::SlotData;
use sp_consensus_aura::sr25519::{AuthorityId as AuraId, AuthorityPair as AuraPair};
use tracing::*;

use crate::configuration::Configuration;

/// Declare an instance of the native executor named `ExecutorDispatch`. Include the wasm binary as
/// the equivalent wasm code.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        humanode_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        humanode_runtime::native_version()
    }
}

/// Executor type.
type Executor = NativeElseWasmExecutor<ExecutorDispatch>;
/// Full node client type.
type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
/// Full node backend type.
type FullBackend = sc_service::TFullBackend<Block>;
/// Full node select chain type.
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
/// Full node Grandpa type.
type FullGrandpa =
    sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;

/// Construct a bare keystore from the configuration.
pub fn keystore_container(
    config: &Configuration,
) -> Result<(KeystoreContainer, TaskManager), ServiceError> {
    let executor = Executor::new(
        config.substrate.wasm_method,
        config.substrate.default_heap_pages,
        config.substrate.max_runtime_instances,
    );

    let (_client, _backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(&config.substrate, None, executor)?;
    Ok((keystore_container, task_manager))
}

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
            FullGrandpa,
            sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
            bioauth_consensus::BioauthBlockImport<
                FullBackend,
                Block,
                FullClient,
                FullGrandpa,
                bioauth_consensus::aura::BlockAuthorExtractor<Block, FullClient, AuraId>,
                bioauth_consensus::api::AuthorizationVerifier<Block, FullClient, AuraId>,
            >,
            SlotDuration,
            Duration,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
> {
    let Configuration {
        substrate: config, ..
    } = config;

    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = Executor::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        Arc::clone(&client),
    );

    let select_chain = sc_consensus::LongestChain::new(Arc::clone(&backend));

    let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
        Arc::clone(&client),
        &(Arc::clone(&client) as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let bioauth_consensus_block_import = bioauth_consensus::BioauthBlockImport::new(
        Arc::clone(&client),
        grandpa_block_import.clone(),
        bioauth_consensus::aura::BlockAuthorExtractor::new(Arc::clone(&client)),
        bioauth_consensus::api::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
    let raw_slot_duration = slot_duration.slot_duration();

    let import_queue =
        sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(ImportQueueParams {
            block_import: bioauth_consensus_block_import.clone(),
            justification_import: Some(Box::new(grandpa_block_import.clone())),
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
            telemetry: telemetry.as_ref().map(|x| x.handle()),
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
            grandpa_block_import,
            grandpa_link,
            bioauth_consensus_block_import,
            slot_duration,
            raw_slot_duration,
            telemetry,
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
        other:
            (
                _grandpa_block_import,
                grandpa_link,
                bioauth_consensus_block_import,
                slot_duration,
                raw_slot_duration,
                mut telemetry,
            ),
    } = new_partial(&config)?;
    let Configuration {
        substrate: mut config,
        bioauth_flow: bioauth_flow_config,
        bioauth_perform_enroll: _,
    } = config;

    config
        .network
        .extra_sets
        .push(sc_finality_grandpa::grandpa_peers_set_config());

    let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
        Arc::clone(&backend),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));

    let bioauth_flow_config = bioauth_flow_config
        .ok_or_else(|| ServiceError::Other("bioauth flow config is not set".into()))?;

    let role = config.role.clone();
    let can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());
    let name = config.network.node_name.clone();
    let keystore = Some(keystore_container.sync_keystore());
    let enable_grandpa = !config.disable_grandpa;
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let prometheus_registry = config.prometheus_registry().cloned();

    let proposer_factory = sc_basic_authorship::ProposerFactory::new(
        task_manager.spawn_handle(),
        Arc::clone(&client),
        Arc::clone(&transaction_pool),
        config.prometheus_registry(),
        telemetry.as_ref().map(|x| x.handle()),
    );

    let proposer_factory = bioauth_consensus::BioauthProposer::new(
        proposer_factory,
        bioauth_consensus::keystore::ValidatorKeyExtractor::new(keystore_container.sync_keystore()),
        bioauth_consensus::api::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: Arc::clone(&client),
            transaction_pool: Arc::clone(&transaction_pool),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync: Some(warp_sync),
        })?;

    let robonode_client = Arc::new(robonode_client::Client {
        base_url: bioauth_flow_config.robonode_url.clone(),
        reqwest: reqwest::Client::new(),
    });

    let rpc_extensions_builder = {
        let client = Arc::clone(&client);
        let pool = Arc::clone(&transaction_pool);
        let robonode_client = Arc::clone(&robonode_client);

        let keystore = keystore_container.keystore();
        let validator_public_key =
            crate::validator_key::AppCryptoPublic::<AuraId>::from_keystore(keystore.as_ref()).await;

        let validator_public_key = match validator_public_key {
            Ok(Some(key)) => {
                info!("Running bioauth flow for {}", key);
                Some(Arc::new(key))
            }
            Ok(None) => {
                warn!("No validator key found, skipping bioauth");
                None
            }
            Err(err) => {
                error!("Keystore returned an error ({}), skipping bioauth", err);
                None
            }
        };

        info!("Bioauth flow starting up");

        let validator_signer = validator_public_key.as_ref().map(|val| {
            Arc::new(crate::validator_key::AppCryptoSigner::new(
                Arc::clone(&keystore),
                Arc::clone(val),
            ))
        });

        Box::new(move |deny_unsafe, _| {
            Ok(humanode_rpc::create(humanode_rpc::Deps {
                client: Arc::clone(&client),
                pool: Arc::clone(&pool),
                deny_unsafe,
                robonode_client: Arc::clone(&robonode_client),
                validator_public_key: validator_public_key.as_ref().map(Arc::clone),
                validator_signer: validator_signer.as_ref().map(Arc::clone),
            }))
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: Arc::clone(&network),
        client: Arc::clone(&client),
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool: Arc::clone(&transaction_pool),
        rpc_extensions_builder,
        backend,
        system_rpc_tx,
        config,
        telemetry: telemetry.as_mut(),
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
            justification_sync_link: Arc::clone(&network),
            block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
            max_block_proposal_slot_portion: None,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        },
    )?;

    // The AURA authoring task is considered essential, i.e. if it
    // fails we take down the service with it.
    task_manager
        .spawn_essential_handle()
        .spawn_blocking("aura", Some("block-authoring"), aura);

    let grandpa_config = sc_finality_grandpa::Config {
        // FIXME #1578 make this available through chainspec.
        // Ref: https://github.com/paritytech/substrate/issues/1578
        gossip_duration: Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role,
        telemetry: None,
    };

    if enable_grandpa {
        let grandpa_config = sc_finality_grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            network,
            voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state: SharedVoterState::empty(),
            telemetry: None,
        };

        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            Some("block-finalization"),
            sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    network_starter.start_network();

    // let mut flow = bioauth_flow::flow::Flow {
    // liveness_data_provider: bioauth_flow::rpc::Provider::new(bioauth_flow_provider_slot),
    // robonode_client,
    // validator_public_key_type: PhantomData,
    // validator_signer_type: PhantomData,
    // };

    let webapp_qrcode = bioauth_flow_config
        .qrcode_params()
        .await
        .and_then(|(webapp_url, rpc_url)| crate::qrcode::WebApp::new(webapp_url, &rpc_url));

    let _render_qr_code = move |prompt: &str| match &webapp_qrcode {
        Ok(ref qrcode) => qrcode.print(),
        Err(ref err) => {
            error!("Bioauth flow - unable to display QR Code: {}", err);
            info!(message = prompt);
        }
    };

    // let bioauth_flow_future = {
    // let client = Arc::clone(&client);
    // let keystore = keystore_container.keystore();
    // let transaction_pool = Arc::clone(&transaction_pool);
    // Box::pin(async move {
    // let validator_public_key =
    // crate::validator_key::AppCryptoPublic::<AuraId>::from_keystore(keystore.as_ref())
    // .await;

    // let validator_public_key = match validator_public_key {
    // Ok(Some(key)) => {
    // info!("Running bioauth flow for {}", key);
    // Arc::new(key)
    // }
    // Ok(None) => {
    // warn!("No validator key found, skipping bioauth");
    // return;
    // }
    // Err(err) => {
    // error!("Keystore returned an error ({}), skipping bioauth", err);
    // return;
    // }
    // };

    // info!("Bioauth flow starting up");

    // let signer = crate::validator_key::AppCryptoSigner::new(
    // Arc::clone(&keystore),
    // Arc::clone(&validator_public_key),
    // );

    // if bioauth_perform_enroll {
    // info!("Bioauth flow - enrolling in progress");

    // render_qr_code("Bioauth flow - waiting for enroll");

    // loop {
    // let result = flow.enroll(validator_public_key.as_ref(), &signer).await;
    // match result {
    // Ok(()) => break,
    // Err(error) => {
    // let (error, retry) = handle_bioauth_error(&error);
    // error!(message = "Bioauth flow - enrollment failure", %error, ?retry);
    // if !retry {
    // panic!("{}", error);
    // }
    // }
    // };
    // }

    // info!("Bioauth flow - enrolling complete");
    // }

    // info!("Bioauth flow - authentication in progress");

    // render_qr_code("Bioauth flow - waiting for authentication");

    // let authenticate_response = loop {
    // let result = flow.authenticate(&signer).await;
    // match result {
    // Ok(v) => break v,
    // Err(error) => {
    // let (error, retry) = handle_bioauth_error(&error);
    // error!(message = "Bioauth flow - authentication failure", %error, ?retry);
    // if !retry {
    // panic!("{}", error);
    // }
    // }
    // };
    // };

    // info!("Bioauth flow - authentication complete");

    // info!(message = "We've obtained an auth ticket", auth_ticket = ?authenticate_response.auth_ticket);

    // let authenticate = pallet_bioauth::Authenticate {
    // ticket: authenticate_response.auth_ticket.into(),
    // ticket_signature: authenticate_response.auth_ticket_signature.into(),
    // };
    // let call = pallet_bioauth::Call::authenticate { req: authenticate };

    // let ext = humanode_runtime::UncheckedExtrinsic::new_unsigned(call.into());

    // let at = client.chain_info().best_hash;
    // transaction_pool
    // .
    // pool()
    // .submit_and_watch(
    // &sp_runtime::generic::BlockId::Hash(at),
    // sp_runtime::transaction_validity::TransactionSource::Local,
    // ext.into(),
    // )
    // .await
    // .unwrap();
    // })
    // };

    // task_manager.spawn_handle().spawn_blocking(
    // "bioauth-flow",
    // Some("bioauth"),
    // bioauth_flow_future,
    // );

    Ok(task_manager)
}
