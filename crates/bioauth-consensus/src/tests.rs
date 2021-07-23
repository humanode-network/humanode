use super::*;
use mockall::predicate::*;
use mockall::*;
use node_primitives::{Block, BlockNumber, Hash, Header};
use pallet_bioauth::StoredAuthTicket;
use sp_api::{ApiError, ApiRef, NativeOrEncoded};
use sp_consensus::BlockOrigin;
use sp_consensus_aura::digests::CompatibleDigestItem;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_aura::Slot;
use sp_core::crypto::Pair;
use sp_runtime::traits::DigestItemFor;
use sp_runtime::Digest;
use std::any::Any;
use std::str::FromStr;

mock! {
    RuntimeApi {
        fn stored_auth_tickets(&self, _at: &sp_api::BlockId<Block>) -> Result<
            NativeOrEncoded<Vec<StoredAuthTicket>>,
            ApiError
        >;

        fn authorities(&self, _at: &sp_api::BlockId<Block>) -> Result<
            NativeOrEncoded<Vec<AuraId>>,
            ApiError
        >;

        fn slot_duration(&self) -> sp_consensus_aura::SlotDuration;
    }
}

#[derive(Clone)]
struct MockWrapperRuntimeApi(Arc<MockRuntimeApi>);

mock! {
    #[derive(Debug)]
    Client {
        async fn check_block(
            &self,
            block: BlockCheckParams<Block>,
        ) -> Result<ImportResult, ConsensusError>;

        async fn import_block(
            &self,
            block: BlockImportParams<Block, TransactionFor<MockClient, Block>>,
            cache: HashMap<sp_consensus::import_queue::CacheKeyId, Vec<u8>>,
        ) -> Result<ImportResult, ConsensusError>;
    }

    impl ProvideRuntimeApi<Block> for Client {
        type Api = MockWrapperRuntimeApi;

        fn runtime_api<'a>(&'a self) -> ApiRef<'a, MockWrapperRuntimeApi>;
    }

    impl HeaderBackend<Block> for Client {
        fn header(&self, _id: sp_api::BlockId<Block>) -> sp_blockchain::Result<Option<Header>>;
        fn info(&self) -> sp_blockchain::Info<Block>;
        fn status(&self, _id: sp_api::BlockId<Block>) -> sp_blockchain::Result<sp_blockchain::BlockStatus>;
        fn number(&self, _hash: Hash) -> sc_service::Result<std::option::Option<BlockNumber>, sp_blockchain::Error>;
        fn hash(&self, _number: sp_api::NumberFor<Block>) -> sp_blockchain::Result<Option<Hash>>;
    }

    impl Finalizer<Block, sc_service::TFullBackend<Block>> for Client {
        fn apply_finality(
            &self,
            _operation: &mut sc_client_api::ClientImportOperation<
                Block,
                sc_service::TFullBackend<Block>,
            >,
            _id: sp_api::BlockId<Block>,
            _justification: Option<sp_runtime::Justification>,
            _notify: bool,
        ) -> sp_blockchain::Result<()>;
        fn finalize_block(
            &self,
            _id: sp_api::BlockId<Block>,
            _justification: Option<sp_runtime::Justification>,
            _notify: bool,
        ) -> sp_blockchain::Result<()>;
    }
}

// mockall doesn't allow implement trait for references inside mock
#[async_trait::async_trait]
impl<'a> BlockImport<Block> for &'a MockClient {
    type Error = ConsensusError;

    type Transaction = TransactionFor<MockClient, Block>;

    async fn check_block(
        &mut self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, ConsensusError> {
        (**self).check_block(block).await
    }

    async fn import_block(
        &mut self,
        block: BlockImportParams<Block, TransactionFor<MockClient, Block>>,
        cache: HashMap<sp_consensus::import_queue::CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, ConsensusError> {
        (**self).import_block(block, cache).await
    }
}

sp_api::mock_impl_runtime_apis! {
    impl BioauthApi<Block> for MockWrapperRuntimeApi {
        #[advanced]
        fn stored_auth_tickets(&self, at: &sp_api::BlockId<Block>) -> Result<
            NativeOrEncoded<Vec<StoredAuthTicket>>,
            ApiError
        > {
            self.0.stored_auth_tickets(at)
        }
    }

    impl AuraApi<Block, AuraId> for MockWrapperRuntimeApi {
        #[advanced]
        fn authorities(&self, at: &sp_api::BlockId<Block>) -> Result<
            NativeOrEncoded<Vec<AuraId>>,
            ApiError
        > {
            self.0.authorities(at)
        }

        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            self.0.slot_duration()
        }
    }
}

#[test]
fn it_denies_block_import_with_error_extract_authorities() {
    let mut mock_client = MockClient::new();
    mock_client
        .expect_info()
        .returning(|| sp_blockchain::Info::<Block> {
            best_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            best_number: 0,
            genesis_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_number: 0,
            number_leaves: 0,
        });

    let mut mock_runtime_api = MockRuntimeApi::new();
    mock_runtime_api.expect_authorities().returning(|_| {
        Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
    });

    let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

    mock_client
        .expect_runtime_api()
        .returning(move || runtime_api.clone().into());

    let client = Arc::new(mock_client);

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
    > = BioauthBlockImport::new(Arc::clone(&client));

    let res = bioauth_block_import.import_block(
        BlockImportParams::new(
            BlockOrigin::Own,
            Header {
                parent_hash: Default::default(),
                number: 1,
                state_root: Default::default(),
                extrinsics_root: Default::default(),
                digest: Digest { logs: vec![] },
            },
        ),
        Default::default(),
    );

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    assert_eq!(
        runtime.block_on(res).unwrap_err().type_id(),
        sp_consensus::Error::Other(Box::new(BioauthBlockImportError::ErrorExtractAuthorities))
            .type_id()
    );
}

#[test]
fn it_denies_block_import_with_invalid_slot_number() {
    let mut mock_client = MockClient::new();
    mock_client
        .expect_info()
        .returning(|| sp_blockchain::Info::<Block> {
            best_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            best_number: 0,
            genesis_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_number: 0,
            number_leaves: 0,
        });

    let mut mock_runtime_api = MockRuntimeApi::new();
    mock_runtime_api.expect_authorities().returning(|_| {
        Ok(NativeOrEncoded::from(vec![
            sp_consensus_aura::sr25519::AuthorityPair::from_string(&format!("//{}", "Alice"), None)
                .expect("static values are valid; qed")
                .public(),
        ]))
    });

    let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

    mock_client
        .expect_runtime_api()
        .returning(move || runtime_api.clone().into());

    let client = Arc::new(mock_client);

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
    > = BioauthBlockImport::new(Arc::clone(&client));

    let res = bioauth_block_import.import_block(
        BlockImportParams::new(
            BlockOrigin::Own,
            Header {
                parent_hash: Default::default(),
                number: 1,
                state_root: Default::default(),
                extrinsics_root: Default::default(),
                digest: Digest { logs: vec![] },
            },
        ),
        Default::default(),
    );

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    assert_eq!(
        runtime.block_on(res).unwrap_err().type_id(),
        sp_consensus::Error::Other(Box::new(BioauthBlockImportError::InvalidSlotNumber)).type_id()
    );
}

#[test]
fn it_denies_block_import_with_error_extract_stored_auth_ticket() {
    let mut mock_client = MockClient::new();
    mock_client
        .expect_info()
        .returning(|| sp_blockchain::Info::<Block> {
            best_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            best_number: 0,
            genesis_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_number: 0,
            number_leaves: 0,
        });

    let mut mock_runtime_api = MockRuntimeApi::new();
    mock_runtime_api.expect_authorities().returning(|_| {
        Ok(NativeOrEncoded::from(vec![
            sp_consensus_aura::sr25519::AuthorityPair::from_string(&format!("//{}", "Alice"), None)
                .expect("static values are valid; qed")
                .public(),
        ]))
    });

    mock_runtime_api
        .expect_stored_auth_tickets()
        .returning(|_| {
            Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
        });

    let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

    mock_client
        .expect_runtime_api()
        .returning(move || runtime_api.clone().into());

    let client = Arc::new(mock_client);

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
    > = BioauthBlockImport::new(Arc::clone(&client));

    let slot = Slot::from(1);
    let digest_item = <DigestItemFor<Block> as CompatibleDigestItem<
        sp_consensus_aura::sr25519::AuthoritySignature,
    >>::aura_pre_digest(slot);

    let res = bioauth_block_import.import_block(
        BlockImportParams::new(
            BlockOrigin::Own,
            Header {
                parent_hash: Default::default(),
                number: 1,
                state_root: Default::default(),
                extrinsics_root: Default::default(),
                digest: Digest {
                    logs: vec![digest_item],
                },
            },
        ),
        Default::default(),
    );

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    assert_eq!(
        runtime.block_on(res).unwrap_err().type_id(),
        sp_consensus::Error::Other(Box::new(
            BioauthBlockImportError::ErrorExtractStoredAuthTickets
        ))
        .type_id()
    );
}

#[test]
fn it_denies_block_import_with_not_bioauth_authorised() {
    let mut mock_client = MockClient::new();
    mock_client
        .expect_info()
        .returning(|| sp_blockchain::Info::<Block> {
            best_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            best_number: 0,
            genesis_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_number: 0,
            number_leaves: 0,
        });

    let mut mock_runtime_api = MockRuntimeApi::new();
    mock_runtime_api.expect_authorities().returning(|_| {
        Ok(NativeOrEncoded::from(vec![
            sp_consensus_aura::sr25519::AuthorityPair::from_string(&format!("//{}", "Alice"), None)
                .expect("static values are valid; qed")
                .public(),
        ]))
    });

    mock_runtime_api
        .expect_stored_auth_tickets()
        .returning(|_| {
            Ok(NativeOrEncoded::from(vec![
                pallet_bioauth::StoredAuthTicket {
                    public_key: "invalid_author".as_bytes().to_vec(),
                    nonce: "1".as_bytes().to_vec(),
                },
            ]))
        });

    let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

    mock_client
        .expect_runtime_api()
        .returning(move || runtime_api.clone().into());

    let client = Arc::new(mock_client);

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
    > = BioauthBlockImport::new(Arc::clone(&client));

    let slot = Slot::from(1);
    let digest_item = <DigestItemFor<Block> as CompatibleDigestItem<
        sp_consensus_aura::sr25519::AuthoritySignature,
    >>::aura_pre_digest(slot);

    let res = bioauth_block_import.import_block(
        BlockImportParams::new(
            BlockOrigin::Own,
            Header {
                parent_hash: Default::default(),
                number: 1,
                state_root: Default::default(),
                extrinsics_root: Default::default(),
                digest: Digest {
                    logs: vec![digest_item],
                },
            },
        ),
        Default::default(),
    );

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    assert_eq!(
        runtime.block_on(res).unwrap_err().type_id(),
        sp_consensus::Error::Other(Box::new(BioauthBlockImportError::NotBioauthAuthorised))
            .type_id()
    );
}

#[test]
fn it_permits_block_import_with_valid_data() {
    let mut mock_client = MockClient::new();
    mock_client
        .expect_info()
        .returning(|| sp_blockchain::Info::<Block> {
            best_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            best_number: 0,
            genesis_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_hash: sp_runtime::testing::H256::from_str(
                "0xf5ef18473d0ee46490ea289ee25ba078febb8bcd9cec752a18d4741a0f24f7ef",
            )
            .unwrap(),
            finalized_number: 0,
            number_leaves: 0,
        });

    let mut mock_runtime_api = MockRuntimeApi::new();
    mock_runtime_api.expect_authorities().returning(|_| {
        Ok(NativeOrEncoded::from(vec![
            sp_consensus_aura::sr25519::AuthorityPair::from_string(&format!("//{}", "Alice"), None)
                .expect("static values are valid; qed")
                .public(),
        ]))
    });

    mock_runtime_api
        .expect_stored_auth_tickets()
        .returning(|_| {
            Ok(NativeOrEncoded::from(vec![
                pallet_bioauth::StoredAuthTicket {
                    public_key: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                        .as_bytes()
                        .to_vec(),
                    nonce: "1".as_bytes().to_vec(),
                },
            ]))
        });

    let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

    mock_client
        .expect_finalize_block()
        .returning(|_, _, _| Ok(()));

    mock_client
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::imported(Default::default())));

    mock_client
        .expect_runtime_api()
        .returning(move || runtime_api.clone().into());

    let client = Arc::new(mock_client);

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
    > = BioauthBlockImport::new(Arc::clone(&client));

    let slot = Slot::from(1);
    let digest_item = <DigestItemFor<Block> as CompatibleDigestItem<
        sp_consensus_aura::sr25519::AuthoritySignature,
    >>::aura_pre_digest(slot);

    let res = bioauth_block_import.import_block(
        BlockImportParams::new(
            BlockOrigin::Own,
            Header {
                parent_hash: Default::default(),
                number: 1,
                state_root: Default::default(),
                extrinsics_root: Default::default(),
                digest: Digest {
                    logs: vec![digest_item],
                },
            },
        ),
        Default::default(),
    );

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    assert_eq!(
        runtime.block_on(res).unwrap(),
        ImportResult::imported(Default::default())
    );
}
