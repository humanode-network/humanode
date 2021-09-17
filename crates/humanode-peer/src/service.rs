//! Initializing, bootstrapping and launching the node from a provided configuration.

#![allow(clippy::type_complexity)]
use std::{marker::PhantomData, sync::Arc, time::Duration};

use futures::prelude::*;
use humanode_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::ExecutorProvider;
use sc_consensus_babe::{self, SlotProportion};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_finality_grandpa::SharedVoterState;
use sc_network::Event;
use sc_service::{Error as ServiceError, KeystoreContainer, PartialComponents, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_runtime::traits::Block as BlockT;
use tracing::*;

use crate::configuration::Configuration;

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
/// Full node GrandpaBlockImport type.
type FullGrandpaBlockImport =
    sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;
/// Full node BabeBlockImport type.
type FullBabeBlockImport =
    sc_consensus_babe::BabeBlockImport<Block, FullClient, FullGrandpaBlockImport>;
/// Full node FullBioauthBlockImport type.
type FullBioauthBlockImport = bioauth_consensus::BioauthBlockImport<
    FullBackend,
    Block,
    FullClient,
    FullBabeBlockImport,
    bioauth_consensus::babe::BlockAuthorExtractor<Block, FullClient>,
    bioauth_consensus::api::AuthorizationVerifier<Block, FullClient, BabeId>,
>;

/// Construct a bare keystore from the configuration.
pub fn keystore_container(
    config: &Configuration,
) -> Result<(KeystoreContainer, TaskManager), ServiceError> {
    let (_client, _backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config.substrate, None)?;
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
            (
                FullBioauthBlockImport,
                sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
                sc_consensus_babe::BabeLink<Block>,
            ),
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

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", worker.run());
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
    let justification_import = grandpa_block_import.clone();

    let (block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::Config::get_or_compute(&*client)?,
        grandpa_block_import,
        Arc::clone(&client),
    )?;

    let block_import = bioauth_consensus::BioauthBlockImport::new(
        Arc::clone(&client),
        block_import,
        bioauth_consensus::babe::BlockAuthorExtractor::new(Arc::clone(&client)),
        bioauth_consensus::api::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let slot_duration = babe_link.config().slot_duration();

    let import_queue = sc_consensus_babe::import_queue(
        babe_link.clone(),
        block_import.clone(),
        Some(Box::new(justification_import)),
        Arc::clone(&client),
        select_chain.clone(),
        move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
                sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
                    *timestamp,
                    slot_duration,
                );

            let uncles =
                sp_authorship::InherentDataProvider::<<Block as BlockT>::Header>::check_inherents();

            Ok((timestamp, slot, uncles))
        },
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let import_setup = (block_import, grandpa_link, babe_link);

    Ok(PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (import_setup, telemetry),
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
        other: (import_setup, mut telemetry),
    } = new_partial(&config)?;
    let Configuration {
        substrate: mut config,
        bioauth_flow: bioauth_flow_config,
        bioauth_perform_enroll,
    } = config;

    let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;

    config
        .network
        .extra_sets
        .push(sc_finality_grandpa::grandpa_peers_set_config());

    let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
        Arc::clone(&backend),
        import_setup.1.shared_authority_set().clone(),
    ));

    let bioauth_flow_config = bioauth_flow_config
        .ok_or_else(|| ServiceError::Other("bioauth flow config is not set".into()))?;

    let role = config.role.clone();
    let name = config.network.node_name.clone();
    let keystore = Some(keystore_container.sync_keystore());
    let enable_grandpa = !config.disable_grandpa;
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let prometheus_registry = config.prometheus_registry().cloned();

    let can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

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
            on_demand: None,
            block_announce_validator_builder: None,
            warp_sync: Some(warp_sync),
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            Arc::clone(&client),
            Arc::clone(&network),
        );
    }

    let robonode_client = Arc::new(robonode_client::Client {
        base_url: bioauth_flow_config.robonode_url.clone(),
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
        telemetry: telemetry.as_mut(),
    })?;

    let (block_import, grandpa_link, babe_link) = import_setup;
    let slot_duration = babe_link.config().slot_duration();
    let client_clone = Arc::clone(&client);

    let babe_config = sc_consensus_babe::BabeParams {
        keystore: keystore_container.sync_keystore(),
        client: Arc::clone(&client),
        select_chain,
        env: proposer_factory,
        block_import,
        sync_oracle: Arc::clone(&network),
        justification_sync_link: Arc::clone(&network),
        create_inherent_data_providers: move |parent, ()| {
            let client_clone = Arc::clone(&client_clone);
            async move {
                let uncles = sc_consensus_uncles::create_uncles_inherent_data_provider(
                    &*client_clone,
                    parent,
                )?;

                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                    sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        slot_duration,
                    );

                Ok((timestamp, slot, uncles))
            }
        },
        force_authoring,
        backoff_authoring_blocks,
        babe_link,
        can_author_with,
        block_proposal_slot_portion: SlotProportion::new(0.5),
        max_block_proposal_slot_portion: None,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
    };

    let babe = sc_consensus_babe::start_babe(babe_config)?;
    task_manager
        .spawn_essential_handle()
        .spawn_blocking("babe-proposer", babe);

    let authority_discovery_role =
        sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());

    let dht_event_stream = network
        .event_stream("authority-discovery")
        .filter_map(|e| async move {
            match e {
                Event::Dht(e) => Some(e),
                _ => None,
            }
        });

    let (authority_discovery_worker, _service) =
        sc_authority_discovery::new_worker_and_service_with_config(
            sc_authority_discovery::WorkerConfig {
                publish_non_global_ips: auth_disc_publish_non_global_ips,
                ..Default::default()
            },
            Arc::clone(&client),
            Arc::clone(&network),
            Box::pin(dht_event_stream),
            authority_discovery_role,
            prometheus_registry.clone(),
        );

    task_manager.spawn_handle().spawn(
        "authority-discovery-worker",
        authority_discovery_worker.run(),
    );

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
            sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    network_starter.start_network();

    let mut flow = bioauth_flow::flow::Flow {
        liveness_data_provider: bioauth_flow::rpc::Provider::new(bioauth_flow_provider_slot),
        robonode_client,
        validator_public_key_type: PhantomData,
        validator_signer_type: PhantomData,
    };

    let webapp_qrcode = bioauth_flow_config
        .qrcode_params()
        .map(|(webapp_url, rpc_url)| {
            crate::qrcode::WebApp::new(webapp_url, rpc_url).map_err(ServiceError::Other)
        })
        .transpose()?;

    let bioauth_flow_future = {
        let client = Arc::clone(&client);
        let keystore = keystore_container.keystore();
        let transaction_pool = Arc::clone(&transaction_pool);
        Box::pin(async move {
            let validator_public_key =
                crate::validator_key::AppCryptoPublic::<BabeId>::from_keystore(keystore.as_ref())
                    .await;

            let validator_public_key = match validator_public_key {
                Ok(Some(key)) => {
                    info!("Running bioauth flow for {}", key);
                    key
                }
                Ok(None) => {
                    warn!("No validator key found, skipping bioauth");
                    return;
                }
                Err(err) => {
                    error!("Keystore returned an error ({}), skipping bioauth", err);
                    return;
                }
            };

            info!("Bioauth flow starting up");

            if bioauth_perform_enroll {
                info!("Bioauth flow - enrolling in progress");

                if let Some(qrcode) = webapp_qrcode.as_ref() {
                    qrcode.print()
                } else {
                    info!("Bioauth flow - waiting for enroll");
                }

                flow.enroll(&validator_public_key)
                    .await
                    .expect("enroll failed");

                info!("Bioauth flow - enrolling complete");
            }

            info!("Bioauth flow - authentication in progress");

            if let Some(qrcode) = webapp_qrcode.as_ref() {
                qrcode.print()
            } else {
                info!("Bioauth flow - waiting for authentication");
            }

            let signer = crate::validator_key::AppCryptoSigner {
                keystore: Arc::clone(&keystore),
                public_key: validator_public_key,
            };

            let authenticate_response = loop {
                let result = flow.authenticate(&signer).await;
                match result {
                    Ok(v) => break v,
                    Err(error) => {
                        error!(message = "Bioauth flow - authentication failure", ?error);
                    }
                };
            };

            info!("Bioauth flow - authentication complete");

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
