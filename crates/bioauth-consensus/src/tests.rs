use super::*;
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
use std::str::FromStr;

#[derive(Default, Clone)]
pub(crate) struct TestApi {
    authorities: Vec<AuraId>,
    stored_auth_tickets: Vec<StoredAuthTicket>,
    error_extract_authorities: bool,
    error_extract_stored_auth_ticket: bool,
}

impl TestApi {
    pub fn new(
        authorities: Vec<AuraId>,
        stored_auth_tickets: Vec<StoredAuthTicket>,
        error_extract_authorities: bool,
        error_extract_stored_auth_ticket: bool,
    ) -> Self {
        TestApi {
            authorities,
            stored_auth_tickets,
            error_extract_authorities,
            error_extract_stored_auth_ticket,
        }
    }
}

pub(crate) struct RuntimeApi {
    inner: TestApi,
}

impl ProvideRuntimeApi<Block> for TestApi {
    type Api = RuntimeApi;

    fn runtime_api<'a>(&'a self) -> ApiRef<'a, Self::Api> {
        RuntimeApi {
            inner: self.clone(),
        }
        .into()
    }
}

impl HeaderMetadata<Block> for TestApi {
    type Error = sp_blockchain::Error;

    fn header_metadata(
        &self,
        _hash: Hash,
    ) -> Result<sp_blockchain::CachedHeaderMetadata<Block>, Self::Error> {
        unimplemented!("Not Required in tests")
    }

    fn insert_header_metadata(
        &self,
        _hash: Hash,
        _header_metadata: sp_blockchain::CachedHeaderMetadata<Block>,
    ) {
        unimplemented!("Not Required in tests")
    }

    fn remove_header_metadata(&self, _hash: Hash) {
        unimplemented!("Not Required in tests")
    }
}

impl HeaderBackend<Block> for TestApi {
    fn header(&self, _id: sp_api::BlockId<Block>) -> sp_blockchain::Result<Option<Header>> {
        unimplemented!("Not Required in tests")
    }

    fn info(&self) -> sp_blockchain::Info<Block> {
        sp_blockchain::Info::<Block> {
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
        }
    }

    fn status(
        &self,
        _id: sp_api::BlockId<Block>,
    ) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
        unimplemented!("Not Required in tests")
    }

    fn number(
        &self,
        _hash: Hash,
    ) -> sc_service::Result<std::option::Option<BlockNumber>, sp_blockchain::Error> {
        unimplemented!("Not Required in tests")
    }

    fn hash(&self, _number: sp_api::NumberFor<Block>) -> sp_blockchain::Result<Option<Hash>> {
        unimplemented!("Not Required in tests")
    }
}

impl LockImportRun<Block, sc_service::TFullBackend<Block>> for TestApi {
    fn lock_import_and_run<R, Err, F>(&self, _f: F) -> Result<R, Err>
    where
        F: FnOnce(
            &mut sc_client_api::ClientImportOperation<Block, sc_service::TFullBackend<Block>>,
        ) -> Result<R, Err>,
        Err: From<sp_blockchain::Error>,
    {
        unimplemented!("Not Required in tests")
    }
}

impl Finalizer<Block, sc_service::TFullBackend<Block>> for TestApi {
    fn apply_finality(
        &self,
        _operation: &mut sc_client_api::ClientImportOperation<
            Block,
            sc_service::TFullBackend<Block>,
        >,
        _id: sp_api::BlockId<Block>,
        _justification: Option<sp_runtime::Justification>,
        _notify: bool,
    ) -> sp_blockchain::Result<()> {
        unimplemented!("Not Required in tests")
    }

    fn finalize_block(
        &self,
        _id: sp_api::BlockId<Block>,
        _justification: Option<sp_runtime::Justification>,
        _notify: bool,
    ) -> sp_blockchain::Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl BlockImport<Block> for TestApi {
    type Error = ConsensusError;

    type Transaction = TransactionFor<TestApi, Block>;

    async fn check_block(
        &mut self,
        _block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        unimplemented!("Not Required in tests")
    }

    async fn import_block(
        &mut self,
        _block: BlockImportParams<Block, Self::Transaction>,
        _cache: HashMap<sp_consensus::import_queue::CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        Ok(ImportResult::imported(Default::default()))
    }
}

#[async_trait::async_trait]
impl<'a> BlockImport<Block> for &'a TestApi {
    type Error = ConsensusError;

    type Transaction = TransactionFor<TestApi, Block>;

    async fn check_block(
        &mut self,
        _block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        unimplemented!("Not Required in tests")
    }

    async fn import_block(
        &mut self,
        _block: BlockImportParams<Block, Self::Transaction>,
        _cache: HashMap<sp_consensus::import_queue::CacheKeyId, Vec<u8>>,
    ) -> Result<ImportResult, Self::Error> {
        Ok(ImportResult::imported(Default::default()))
    }
}

sp_api::mock_impl_runtime_apis! {
    impl BioauthApi<Block> for RuntimeApi {
        #[advanced]
        fn stored_auth_tickets(&self, _at: &sp_api::BlockId<Block>) -> Result<
            NativeOrEncoded<Vec<StoredAuthTicket>>,
            ApiError
        > {
            if self.inner.error_extract_stored_auth_ticket {
                Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
            } else {
                Ok(self.inner.stored_auth_tickets.clone().into())
            }
        }
    }

    impl AuraApi<Block, AuraId> for RuntimeApi {
        #[advanced]
        fn authorities(&self, _at: &sp_api::BlockId<Block>) -> Result<
            NativeOrEncoded<Vec<AuraId>>,
            ApiError
        > {
            if self.inner.error_extract_authorities {
                Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
            } else {
                Ok(self.inner.authorities.clone().into())
            }
        }

        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            unimplemented!("Not Required in tests")
        }
    }
}

#[test]
fn it_works() {
    let test_api = Arc::new(TestApi::new(
        vec![sp_consensus_aura::sr25519::AuthorityPair::from_string(
            &format!("//{}", "Alice"),
            None,
        )
        .expect("static values are valid; qed")
        .public()],
        vec![pallet_bioauth::StoredAuthTicket {
            public_key: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                .as_bytes()
                .to_vec(),
            nonce: "1".as_bytes().to_vec(),
        }],
        false,
        false,
    ));

    let mut bioauth_block_import: BioauthBlockImport<sc_service::TFullBackend<Block>, _, TestApi> =
        BioauthBlockImport::new(Arc::clone(&test_api));

    let slot = Slot::from(0);
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
    // let res1 = runtime.block_on(res);
    // print!("{:?}", res1);

    // assert_eq!(2 + 2, 4);
}

#[test]
fn it_denies_block_import_with_not_bioauth_authorised() {
    let test_api = Arc::new(TestApi::new(
        vec![sp_consensus_aura::sr25519::AuthorityPair::from_string(
            &format!("//{}", "Alice"),
            None,
        )
        .expect("static values are valid; qed")
        .public()],
        vec![pallet_bioauth::StoredAuthTicket {
            public_key: "invalid_author".as_bytes().to_vec(),
            nonce: "1".as_bytes().to_vec(),
        }],
        false,
        false,
    ));

    let mut bioauth_block_import: BioauthBlockImport<sc_service::TFullBackend<Block>, _, TestApi> =
        BioauthBlockImport::new(Arc::clone(&test_api));

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

    // We need to use to_string due to Eq, kind() doesn't implemented for sp_consensus::Error
    let expected_error_string =
        sp_consensus::Error::Other(Box::new(BioauthBlockImportError::NotBioauthAuthorised))
            .to_string();
    let actual_error_string = runtime.block_on(res).unwrap_err().to_string();

    assert_eq!(actual_error_string, expected_error_string);
}

#[test]
fn it_denies_block_import_with_error_extract_authorities() {
    let test_api = Arc::new(TestApi::new(
        vec![sp_consensus_aura::sr25519::AuthorityPair::from_string(
            &format!("//{}", "Alice"),
            None,
        )
        .expect("static values are valid; qed")
        .public()],
        vec![pallet_bioauth::StoredAuthTicket {
            public_key: "invalid_author".as_bytes().to_vec(),
            nonce: "1".as_bytes().to_vec(),
        }],
        true,
        false,
    ));

    let mut bioauth_block_import: BioauthBlockImport<sc_service::TFullBackend<Block>, _, TestApi> =
        BioauthBlockImport::new(Arc::clone(&test_api));

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

    // We need to use to_string due to Eq, kind() doesn't implemented for sp_consensus::Error
    let expected_error_string =
        sp_consensus::Error::Other(Box::new(BioauthBlockImportError::ErrorExtractAuthorities))
            .to_string();
    let actual_error_string = runtime.block_on(res).unwrap_err().to_string();

    assert_eq!(actual_error_string, expected_error_string);
}

#[test]
fn it_denies_block_import_with_invalid_slot_number() {
    let test_api = Arc::new(TestApi::new(
        vec![sp_consensus_aura::sr25519::AuthorityPair::from_string(
            &format!("//{}", "Alice"),
            None,
        )
        .expect("static values are valid; qed")
        .public()],
        vec![pallet_bioauth::StoredAuthTicket {
            public_key: "invalid_author".as_bytes().to_vec(),
            nonce: "1".as_bytes().to_vec(),
        }],
        false,
        false,
    ));

    let mut bioauth_block_import: BioauthBlockImport<sc_service::TFullBackend<Block>, _, TestApi> =
        BioauthBlockImport::new(Arc::clone(&test_api));

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

    // We need to use to_string due to Eq, kind() doesn't implemented for sp_consensus::Error
    let expected_error_string =
        sp_consensus::Error::Other(Box::new(BioauthBlockImportError::InvalidSlotNumber))
            .to_string();
    let actual_error_string = runtime.block_on(res).unwrap_err().to_string();

    assert_eq!(actual_error_string, expected_error_string);
}

#[test]
fn it_denies_block_import_with_error_extract_stored_auth_ticket() {
    let test_api = Arc::new(TestApi::new(
        vec![sp_consensus_aura::sr25519::AuthorityPair::from_string(
            &format!("//{}", "Alice"),
            None,
        )
        .expect("static values are valid; qed")
        .public()],
        vec![pallet_bioauth::StoredAuthTicket {
            public_key: "invalid_author".as_bytes().to_vec(),
            nonce: "1".as_bytes().to_vec(),
        }],
        false,
        true,
    ));

    let mut bioauth_block_import: BioauthBlockImport<sc_service::TFullBackend<Block>, _, TestApi> =
        BioauthBlockImport::new(Arc::clone(&test_api));

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

    // We need to use to_string due to Eq, kind() doesn't implemented for sp_consensus::Error
    let expected_error_string = sp_consensus::Error::Other(Box::new(
        BioauthBlockImportError::ErrorExtractStoredAuthTickets,
    ))
    .to_string();
    let actual_error_string = runtime.block_on(res).unwrap_err().to_string();

    assert_eq!(actual_error_string, expected_error_string);
}
