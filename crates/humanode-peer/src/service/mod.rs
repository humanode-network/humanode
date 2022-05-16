//! Initializing, bootstrapping and launching the node from a provided configuration.

#![allow(clippy::type_complexity)]
use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Duration,
};

use fc_consensus::FrontierBlockImport;
use fc_mapping_sync::{MappingSyncWorker, SyncStrategy};
use fc_rpc::EthTask;
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use futures::StreamExt;
use humanode_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::{BlockchainEvents, ExecutorProvider};
use sc_consensus_babe::SlotProportion;
pub use sc_executor::NativeElseWasmExecutor;
use sc_finality_grandpa::SharedVoterState;
use sc_service::{Error as ServiceError, KeystoreContainer, PartialComponents, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus_babe::AuthorityId as BabeId;
use tracing::*;

use crate::configuration::Configuration;

pub mod frontier;
pub mod inherents;

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

/// Keystore bioauth identifier used at the keystore.
pub type KeystoreBioauthId = keystore_bioauth_account_id::KeystoreBioauthAccountId;
/// Executor type.
type Executor = NativeElseWasmExecutor<ExecutorDispatch>;
/// Full node client type.
type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
/// Full node backend type.
type FullBackend = sc_service::TFullBackend<Block>;
/// Full node select chain type.
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
/// Full type for GrandpaBlockImport.
type FullGrandpa =
    sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;
/// Full type for BabeBlockImport.
type FullBabe = sc_consensus_babe::BabeBlockImport<Block, FullClient, FullGrandpa>;
/// Full type for FrontierBlockImport.
type FullFrontier = FrontierBlockImport<Block, FullBabe, FullClient>;
/// Full type for BioauthBlockImport.
type FullBioauth = bioauth_consensus::BioauthBlockImport<
    FullBackend,
    Block,
    FullClient,
    FullFrontier,
    bioauth_consensus::babe::BlockAuthorExtractor<Block, FullClient>,
    bioauth_consensus::api::AuthorizationVerifier<Block, FullClient, BabeId>,
>;
/// Frontier backend type.
type FrontierBackend = fc_db::Backend<Block>;

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
            sc_consensus_babe::BabeLink<Block>,
            FullBioauth,
            Duration,
            inherents::Creator,
            Arc<FrontierBackend>,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
> {
    let Configuration {
        substrate: config,
        evm: evm_config,
        ..
    } = config;

    let evm_config = evm_config
        .as_ref()
        .ok_or_else(|| ServiceError::Other("evm config is not set".into()))?;

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

    let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::Config::get_or_compute(&*client)?,
        grandpa_block_import.clone(),
        Arc::clone(&client),
    )?;

    let frontier_backend = Arc::new(frontier::open_backend(config)?);
    let frontier_block_import = FrontierBlockImport::new(
        babe_block_import,
        Arc::clone(&client),
        Arc::clone(&frontier_backend),
    );

    let bioauth_consensus_block_import = bioauth_consensus::BioauthBlockImport::new(
        Arc::clone(&client),
        frontier_block_import,
        bioauth_consensus::babe::BlockAuthorExtractor::new(Arc::clone(&client)),
        bioauth_consensus::api::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let raw_slot_duration = babe_link.config().slot_duration();
    let eth_target_gas_price = evm_config.target_gas_price;
    let inherent_data_providers_creator = inherents::Creator {
        raw_slot_duration,
        eth_target_gas_price,
    };

    let import_queue = sc_consensus_babe::import_queue(
        babe_link.clone(),
        bioauth_consensus_block_import.clone(),
        Some(Box::new(grandpa_block_import.clone())),
        Arc::clone(&client),
        select_chain.clone(),
        inherent_data_providers_creator.clone(),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
        telemetry.as_ref().map(|x| x.handle()),
    )?;
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
            babe_link,
            bioauth_consensus_block_import,
            raw_slot_duration,
            inherent_data_providers_creator,
            frontier_backend,
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
                babe_link,
                bioauth_consensus_block_import,
                raw_slot_duration,
                inherent_data_providers_creator,
                frontier_backend,
                mut telemetry,
            ),
    } = new_partial(&config)?;
    let Configuration {
        substrate: mut config,
        bioauth_flow: bioauth_flow_config,
        bioauth_perform_enroll,
        evm: evm_config,
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

    let evm_config =
        evm_config.expect("already used during substrate partial components exctraction");

    let role = config.role.clone();
    let can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());
    let name = config.network.node_name.clone();
    let keystore = Some(keystore_container.sync_keystore());
    let enable_grandpa = !config.disable_grandpa;
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let prometheus_registry = config.prometheus_registry().cloned();
    let eth_filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
    let eth_fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));
    let eth_fee_history_limit = evm_config.fee_history_limit;
    let eth_overrides = humanode_rpc::overrides_handle(Arc::clone(&client));

    let proposer_factory = sc_basic_authorship::ProposerFactory::new(
        task_manager.spawn_handle(),
        Arc::clone(&client),
        Arc::clone(&transaction_pool),
        config.prometheus_registry(),
        telemetry.as_ref().map(|x| x.handle()),
    );

    let validator_key_extractor = Arc::new(
        bioauth_consensus::keystore::ValidatorKeyExtractor::new(keystore_container.sync_keystore()),
    );

    let proposer_factory = bioauth_consensus::BioauthProposer::new(
        proposer_factory,
        Arc::clone(&validator_key_extractor),
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
        let bioauth_validator_key_extractor =
            Arc::new(bioauth_consensus::keystore::ValidatorKeyExtractor::<
                KeystoreBioauthId,
            >::new(keystore_container.sync_keystore()));
        let bioauth_validator_signer_factory = {
            let keystore = keystore_container.keystore();
            Arc::new(move |key| {
                crate::validator_key::AppCryptoSigner::new(
                    Arc::clone(&keystore),
                    crate::validator_key::AppCryptoPublic(key),
                )
            })
        };
        let network = Arc::clone(&network);

        let grandpa_justification_stream = grandpa_link.justification_stream();
        let grandpa_shared_authority_set = grandpa_link.shared_authority_set().clone();
        let grandpa_shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();
        let grandpa_finality_proof_provider =
            sc_finality_grandpa::FinalityProofProvider::new_for_service(
                Arc::clone(&backend),
                Some(grandpa_shared_authority_set.clone()),
            );

        let babe_config = babe_link.config().clone();
        let babe_shared_epoch_changes = babe_link.epoch_changes().clone();

        let keystore = keystore_container.sync_keystore();
        let select_chain = select_chain.clone();

        let eth_filter_pool = eth_filter_pool.clone();
        let eth_max_stored_filters = evm_config.max_stored_filters;
        let frontier_backend = Arc::clone(&frontier_backend);
        let eth_overrides = Arc::clone(&eth_overrides);
        let eth_block_data_cache = Arc::new(fc_rpc::EthBlockDataCache::new(
            task_manager.spawn_handle(),
            Arc::clone(&eth_overrides),
            50,
            50,
        ));
        let eth_max_past_logs = evm_config.max_past_logs;
        let eth_fee_history_cache = Arc::clone(&eth_fee_history_cache);

        let subscription_task_executor = Arc::new(sc_rpc::SubscriptionTaskExecutor::new(
            task_manager.spawn_handle(),
        ));

        Box::new(move |deny_unsafe, _| {
            Ok(humanode_rpc::create(humanode_rpc::Deps {
                client: Arc::clone(&client),
                pool: Arc::clone(&pool),
                deny_unsafe,
                graph: Arc::clone(pool.pool()),
                network: Arc::clone(&network),
                author_ext: humanode_rpc::AuthorExtDeps {
                    author_validator_key_extractor: Arc::clone(&bioauth_validator_key_extractor),
                },
                bioauth: humanode_rpc::BioauthDeps {
                    robonode_client: Arc::clone(&robonode_client),
                    bioauth_flow_slot: Arc::clone(&bioauth_flow_rpc_slot),
                    bioauth_validator_signer_factory: Arc::clone(&bioauth_validator_signer_factory),
                    bioauth_validator_key_extractor: Arc::clone(&bioauth_validator_key_extractor),
                },
                babe: humanode_rpc::BabeDeps {
                    babe_config: babe_config.clone(),
                    babe_shared_epoch_changes: babe_shared_epoch_changes.clone(),
                    keystore: Arc::clone(&keystore),
                },
                grandpa: humanode_rpc::GrandpaDeps {
                    grandpa_shared_voter_state: grandpa_shared_voter_state.clone(),
                    grandpa_shared_authority_set: grandpa_shared_authority_set.clone(),
                    grandpa_justification_stream: grandpa_justification_stream.clone(),
                    grandpa_finality_provider: Arc::clone(&grandpa_finality_proof_provider),
                },
                select_chain: select_chain.clone(),
                evm: humanode_rpc::EvmDeps {
                    eth_filter_pool: eth_filter_pool.clone(),
                    eth_max_stored_filters,
                    eth_backend: Arc::clone(&frontier_backend),
                    eth_max_past_logs,
                    eth_fee_history_limit,
                    eth_fee_history_cache: Arc::clone(&eth_fee_history_cache),
                    eth_overrides: Arc::clone(&eth_overrides),
                    eth_block_data_cache: Arc::clone(&eth_block_data_cache),
                },
                subscription_task_executor: Arc::clone(&subscription_task_executor),
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
        backend: Arc::clone(&backend),
        system_rpc_tx,
        config,
        telemetry: telemetry.as_mut(),
    })?;

    let babe_config = sc_consensus_babe::BabeParams {
        keystore: keystore_container.sync_keystore(),
        client: Arc::clone(&client),
        select_chain,
        env: proposer_factory,
        block_import: bioauth_consensus_block_import,
        sync_oracle: Arc::clone(&network),
        justification_sync_link: Arc::clone(&network),
        create_inherent_data_providers: inherent_data_providers_creator,
        force_authoring,
        backoff_authoring_blocks,
        babe_link,
        can_author_with,
        block_proposal_slot_portion: SlotProportion::new(0.5),
        max_block_proposal_slot_portion: None,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
    };

    let babe = sc_consensus_babe::start_babe(babe_config)?;
    task_manager.spawn_essential_handle().spawn_blocking(
        "babe-proposer",
        Some("block-authoring"),
        babe,
    );

    let grandpa_config = sc_finality_grandpa::Config {
        // See substrate#1578: make this available through chainspec.
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

    task_manager.spawn_essential_handle().spawn_blocking(
        "frontier-mapping-sync-worker",
        Some("evm"),
        MappingSyncWorker::new(
            client.import_notification_stream(),
            raw_slot_duration,
            Arc::clone(&client),
            backend,
            Arc::clone(&frontier_backend),
            SyncStrategy::Normal,
        )
        .for_each(|()| futures::future::ready(())),
    );

    // Spawn Frontier FeeHistory cache maintenance task.
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("evm"),
        EthTask::fee_history_task(
            Arc::clone(&client),
            Arc::clone(&eth_overrides),
            eth_fee_history_cache,
            eth_fee_history_limit,
        ),
    );

    task_manager.spawn_essential_handle().spawn(
        "frontier-schema-cache-task",
        Some("evm"),
        EthTask::ethereum_schema_cache_task(Arc::clone(&client), frontier_backend),
    );

    // Spawn Frontier EthFilterApi maintenance task.
    if let Some(eth_filter_pool) = eth_filter_pool {
        /// Each filter is allowed to stay in the pool for 100 blocks.
        const FILTER_RETAIN_THRESHOLD: u64 = 100;
        task_manager.spawn_essential_handle().spawn(
            "frontier-filter-pool",
            Some("evm"),
            EthTask::filter_pool_task(
                Arc::clone(&client),
                eth_filter_pool,
                FILTER_RETAIN_THRESHOLD,
            ),
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
        .await
        .and_then(|(webapp_url, rpc_url)| crate::qrcode::WebApp::new(webapp_url, &rpc_url));

    let render_qr_code = move |prompt: &str| match &webapp_qrcode {
        Ok(ref qrcode) => qrcode.print(),
        Err(ref err) => {
            error!("Bioauth flow - unable to display QR Code: {}", err);
            info!(message = prompt);
        }
    };

    let bioauth_flow_future = {
        let client = Arc::clone(&client);
        let keystore = keystore_container.keystore();
        let transaction_pool = Arc::clone(&transaction_pool);
        Box::pin(async move {
            let validator_public_key =
                crate::validator_key::AppCryptoPublic::<KeystoreBioauthId>::from_keystore(
                    keystore.as_ref(),
                )
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

            let signer = crate::validator_key::AppCryptoSigner::new(
                Arc::clone(&keystore),
                validator_public_key.clone(),
            );

            if bioauth_perform_enroll {
                info!("Bioauth flow - enrolling in progress");

                render_qr_code("Bioauth flow - waiting for enroll");

                loop {
                    let result = flow.enroll(&validator_public_key, &signer).await;
                    match result {
                        Ok(()) => break,
                        Err(error) => {
                            let (error, retry) = handle_bioauth_error(&error);
                            error!(message = "Bioauth flow - enrollment failure", %error, ?retry);
                            if !retry {
                                panic!("{}", error);
                            }
                        }
                    };
                }

                info!("Bioauth flow - enrolling complete");
            }

            info!("Bioauth flow - authentication in progress");

            render_qr_code("Bioauth flow - waiting for authentication");

            let authenticate_response = loop {
                let result = flow.authenticate(&signer).await;
                match result {
                    Ok(v) => break v,
                    Err(error) => {
                        let (error, retry) = handle_bioauth_error(&error);
                        error!(message = "Bioauth flow - authentication failure", %error, ?retry);
                        if !retry {
                            panic!("{}", error);
                        }
                    }
                };
            };

            info!("Bioauth flow - authentication complete");

            info!(message = "We've obtained an auth ticket", auth_ticket = ?authenticate_response.auth_ticket);

            let authenticate = pallet_bioauth::Authenticate {
                ticket: authenticate_response.auth_ticket.into(),
                ticket_signature: authenticate_response.auth_ticket_signature.into(),
            };
            let call = pallet_bioauth::Call::authenticate { req: authenticate };

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

    task_manager.spawn_handle().spawn_blocking(
        "bioauth-flow",
        Some("bioauth"),
        bioauth_flow_future,
    );

    Ok(task_manager)
}

/// Handle the bioauth error in a user-friendly way.
fn handle_bioauth_error(error: &anyhow::Error) -> (String, bool) {
    use robonode_client::{AuthenticateError, EnrollError, Error};

    let face_scan_rejected_message = "the face scan was rejected, this is likely caused by a failed liveness check, so please try again; changing lighting conditions or using a different phone can help";

    if let Some(error) = error.downcast_ref::<Error<EnrollError>>() {
        match error {
            Error::Call(EnrollError::PersonAlreadyEnrolled) => {
                ("you have already enrolled".to_owned(), false)
            }
            Error::Call(EnrollError::PublicKeyAlreadyUsed) => (
                "the validator key you supplied was already used".to_owned(),
                false,
            ),
            Error::Call(EnrollError::FaceScanRejected) => {
                (face_scan_rejected_message.to_owned(), true)
            }
            Error::Call(EnrollError::InvalidLivenessData) => {
                ("the provided liveness data was invalid".to_owned(), true)
            }
            Error::Call(EnrollError::InvalidPublicKey) => {
                ("the public key was invalid".to_owned(), false)
            }
            Error::Call(EnrollError::LogicInternal) => {
                ("an internal logic error has occured".to_owned(), true)
            }
            Error::Call(EnrollError::UnknownCode(error_code)) => (
                format!(
                    "an unknown error code received from the server: {}",
                    error_code
                ),
                false,
            ),
            Error::Call(EnrollError::Unknown(err)) => (err.clone(), true),
            Error::Reqwest(err) => (err.to_string(), err.is_timeout()),
        }
    } else if let Some(error) = error.downcast_ref::<Error<AuthenticateError>>() {
        match error {
            Error::Call(AuthenticateError::InvalidLivenessData) => {
                ("the provided liveness data was invalid".to_owned(), true)
            }
            Error::Call(AuthenticateError::PersonNotFound) => (
                "we were unable to find you in the system; have you already enrolled?".to_owned(),
                true,
            ),
            Error::Call(AuthenticateError::FaceScanRejected) => {
                (face_scan_rejected_message.to_owned(), true)
            }
            Error::Call(AuthenticateError::SignatureInvalid) => {
                ("the validator key used for authentication does not match the one used during enroll; you have likely used a different mnemonic, but you have to use the same one, otherwise you will be unable to authenticate; you have saved the mnemonic somewhere as requested, right? ;) if you've lost your menmonic you will be unable to continue.".to_owned(), true)
            }
            Error::Call(AuthenticateError::LogicInternal) => {
                ("an internal logic error has occured".to_owned(), true)
            }
            Error::Call(AuthenticateError::UnknownCode(error_code)) => (
                format!(
                    "an unknown error code received from the server: {}",
                    error_code
                ),
                false,
            ),
            Error::Call(AuthenticateError::Unknown(err)) => (err.clone(), true),
            Error::Reqwest(err) => (err.to_string(), err.is_timeout()),
        }
    } else {
        (error.to_string(), false)
    }
}
