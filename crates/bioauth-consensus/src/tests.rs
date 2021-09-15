use mockall::predicate::*;
use mockall::*;
use node_primitives::{Block, BlockNumber, Hash, Header};
use pallet_bioauth::{self, BioauthApi, StoredAuthTicket};
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{ApiError, ApiRef, NativeOrEncoded, ProvideRuntimeApi, TransactionFor};
use sp_application_crypto::Pair;
use sp_blockchain::{well_known_cache_keys, HeaderBackend};
use sp_consensus::{BlockOrigin, Error as ConsensusError};
use sp_consensus_aura::{
    digests::CompatibleDigestItem, sr25519::AuthorityId as AuraId, AuraApi, Slot,
};
use sp_runtime::{traits::DigestItemFor, Digest};
use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{BioauthBlockImport, BioauthBlockImportError};

type MockValidatorPublicKey = AuraId;

type MockAuthTicket = StoredAuthTicket<MockValidatorPublicKey>;

mock! {
    RuntimeApi {
        fn stored_auth_tickets(&self, _at: &sp_api::BlockId<Block>) -> Result<
            NativeOrEncoded<Vec<MockAuthTicket>>,
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
            cache: HashMap<well_known_cache_keys::Id, Vec<u8>>,
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
}

mock! {
    BlockImportWrapper {}

    #[async_trait::async_trait]
    impl BlockImport<Block> for BlockImportWrapper {
        type Error = ConsensusError;
        type Transaction = TransactionFor<MockClient, Block>;

        async fn check_block(
            &mut self,
            block: BlockCheckParams<Block>,
        ) -> Result<ImportResult, ConsensusError>;

        async fn import_block(
            &mut self,
            block: BlockImportParams<Block, TransactionFor<MockClient, Block>>,
            cache: HashMap<well_known_cache_keys::Id, Vec<u8>>,
        ) -> Result<ImportResult, ConsensusError>;
    }
}

sp_api::mock_impl_runtime_apis! {
    impl BioauthApi<Block, MockValidatorPublicKey> for MockWrapperRuntimeApi {
        #[advanced]
        fn stored_auth_tickets(&self, at: &sp_api::BlockId<Block>) -> Result<
            NativeOrEncoded<Vec<MockAuthTicket>>,
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

fn prepare_get_info() -> sp_blockchain::Info<Block> {
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
        finalized_state: None,
        number_leaves: 0,
    }
}

fn prepare_block_import_with_aura_pre_digest(
    empty_digest: bool,
) -> BlockImportParams<Block, TransactionFor<MockClient, Block>> {
    let mut digest_items = vec![];
    if !empty_digest {
        let slot = Slot::from(1);
        let item = <DigestItemFor<Block> as CompatibleDigestItem<
            sp_consensus_aura::sr25519::AuthoritySignature,
        >>::aura_pre_digest(slot);
        digest_items.push(item);
    }

    BlockImportParams::new(
        BlockOrigin::Own,
        Header {
            parent_hash: Default::default(),
            number: 1,
            state_root: Default::default(),
            extrinsics_root: Default::default(),
            digest: Digest { logs: digest_items },
        },
    )
}

fn extract_bioauth_err(
    err: &sp_consensus::Error,
) -> &BioauthBlockImportError<
    crate::aura::AuraBlockAuthorExtractorError,
    crate::bioauth::AuraAuthorizationVerifierError,
> {
    if let sp_consensus::Error::Other(boxed_err) = err {
        if let Some(raw_err) = boxed_err.downcast_ref::<BioauthBlockImportError<
            crate::aura::AuraBlockAuthorExtractorError,
            crate::bioauth::AuraAuthorizationVerifierError,
        >>() {
            return raw_err;
        }
    }
    panic!("Unexpected consensus error: {}", err);
}

macro_rules! assert_consensus_error {
    ($expression:expr, $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        {
            let err_hold = $expression;
            let err = extract_bioauth_err(&err_hold);
            match err {
                $( $pattern )|+ $( if $guard )? => (),
                ref e => panic!(
                    "assertion failed: `{:?}` does not match `{}`",
                    e, stringify!($( $pattern )|+ $( if $guard )?)
                )
            }
        }
    }
}

#[tokio::test]
async fn it_denies_block_import_with_error_extract_authorities() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

    let mut mock_runtime_api = MockRuntimeApi::new();
    mock_runtime_api.expect_authorities().returning(|_| {
        Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
    });

    let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

    mock_client
        .expect_runtime_api()
        .returning(move || runtime_api.clone().into());

    let client = Arc::new(mock_client);

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
        MockBlockImportWrapper,
        _,
        _,
    > = BioauthBlockImport::new(
        Arc::clone(&client),
        block_import,
        crate::aura::BlockAuthorExtractor::new(Arc::clone(&client)),
        crate::bioauth::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let res = bioauth_block_import
        .import_block(
            prepare_block_import_with_aura_pre_digest(true),
            Default::default(),
        )
        .await;

    assert_consensus_error!(
        res.unwrap_err(),
        BioauthBlockImportError::BlockAuthorExtraction(
            crate::aura::AuraBlockAuthorExtractorError::UnableToExtractAuthorities(err),
        ) if err.to_string() == "Test error",
    );
}

#[tokio::test]
async fn it_denies_block_import_with_invalid_slot_number() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

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

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
        MockBlockImportWrapper,
        _,
        _,
    > = BioauthBlockImport::new(
        Arc::clone(&client),
        block_import,
        crate::aura::BlockAuthorExtractor::new(Arc::clone(&client)),
        crate::bioauth::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let res = bioauth_block_import
        .import_block(
            prepare_block_import_with_aura_pre_digest(true),
            Default::default(),
        )
        .await;

    assert_consensus_error!(
        res.unwrap_err(),
        BioauthBlockImportError::BlockAuthorExtraction(_)
    );
}

#[tokio::test]
async fn it_denies_block_import_with_error_extract_stored_auth_ticket() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

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

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
        MockBlockImportWrapper,
        _,
        _,
    > = BioauthBlockImport::new(
        Arc::clone(&client),
        block_import,
        crate::aura::BlockAuthorExtractor::new(Arc::clone(&client)),
        crate::bioauth::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let res = bioauth_block_import
        .import_block(
            prepare_block_import_with_aura_pre_digest(false),
            Default::default(),
        )
        .await;

    assert_consensus_error!(
        res.unwrap_err(),
        BioauthBlockImportError::AuthorizationVerifier(
            crate::bioauth::AuraAuthorizationVerifierError::UnableToExtractStoredAuthTickets(_)
        ),
    );
}

#[tokio::test]
async fn it_denies_block_import_with_not_bioauth_authorized() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

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
            Ok(NativeOrEncoded::from(vec![MockAuthTicket {
                public_key: sp_consensus_aura::sr25519::AuthorityPair::from_string(
                    &format!("//{}", "Bob"),
                    None,
                )
                .expect("static values are valid; qed")
                .public(),
                nonce: b"1".to_vec(),
            }]))
        });

    let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

    mock_client
        .expect_runtime_api()
        .returning(move || runtime_api.clone().into());

    let client = Arc::new(mock_client);

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
        MockBlockImportWrapper,
        _,
        _,
    > = BioauthBlockImport::new(
        Arc::clone(&client),
        block_import,
        crate::aura::BlockAuthorExtractor::new(Arc::clone(&client)),
        crate::bioauth::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let res = bioauth_block_import
        .import_block(
            prepare_block_import_with_aura_pre_digest(false),
            Default::default(),
        )
        .await;

    assert_consensus_error!(
        res.unwrap_err(),
        BioauthBlockImportError::NotBioauthAuthorized,
    );
}

#[tokio::test]
async fn it_permits_block_import_with_valid_data() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

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
            Ok(NativeOrEncoded::from(vec![MockAuthTicket {
                public_key: sp_consensus_aura::sr25519::AuthorityPair::from_string(
                    &format!("//{}", "Alice"),
                    None,
                )
                .expect("static values are valid; qed")
                .public(),
                nonce: b"1".to_vec(),
            }]))
        });

    let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

    mock_client
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::imported(Default::default())));

    mock_client
        .expect_runtime_api()
        .returning(move || runtime_api.clone().into());

    let client = Arc::new(mock_client);

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        MockClient,
        MockBlockImportWrapper,
        _,
        _,
    > = BioauthBlockImport::new(
        Arc::clone(&client),
        block_import,
        crate::aura::BlockAuthorExtractor::new(Arc::clone(&client)),
        crate::bioauth::AuthorizationVerifier::new(Arc::clone(&client)),
    );

    let res = bioauth_block_import
        .import_block(
            prepare_block_import_with_aura_pre_digest(false),
            Default::default(),
        )
        .await;

    assert_eq!(res.unwrap(), ImportResult::imported(Default::default()));
}
