use mockall::predicate::*;
use mockall::*;
use node_primitives::{Block, BlockNumber, Hash, Header};
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{ApiRef, ProvideRuntimeApi, TransactionFor};
use sp_blockchain::{well_known_cache_keys, HeaderBackend};
use sp_consensus::{BlockOrigin, Error as ConsensusError};
use sp_runtime::{traits::Block as BlockT, Digest};
use std::{collections::HashMap, sync::Arc};

use crate::{BioauthBlockImport, BioauthBlockImportError};

type MockPublicKeyType = ();
type MockRuntimeApi = ();

#[derive(Clone)]
struct MockWrapperRuntimeApi(Arc<MockRuntimeApi>);

/// The Fake trait is used to reuse required implementations from mock_impl_runtime_apis
/// macro that should be used in impl ProvideRuntimeApi.
trait MockFakeTrait<Block> {}

sp_api::mock_impl_runtime_apis! {
    impl MockFakeTrait<Block> for MockWrapperRuntimeApi {}
}

mock! {
    Client {}

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

/// BioauthBlockImport requires impl std::error::Error.
#[derive(Debug, thiserror::Error)]
pub enum MockBlockAuthorExtractorError {
    #[error("block author error")]
    BlockAuthorExtractorError,
}

mock! {
    BlockAuthorExtractor {}

    impl crate::BlockAuthorExtractor for BlockAuthorExtractor {
        type Error = MockBlockAuthorExtractorError;
        type Block = Block;
        type PublicKeyType = MockPublicKeyType;

        fn extract_block_author(
            &self,
            at: &sp_api::BlockId<Block>,
            block_header: &<Block as BlockT>::Header,
        ) -> Result<MockPublicKeyType, MockBlockAuthorExtractorError>;
    }

    impl Clone for BlockAuthorExtractor {
        fn clone(&self) -> MockBlockAuthorExtractor;
    }
}

/// BioauthBlockImport requires impl std::error::Error.
#[derive(Debug, thiserror::Error)]
pub enum MockAuthorizationVerifierError {
    #[error("authorization error")]
    AuthorizationVerifierError,
}

mock! {
    AuthorizationVerifier {}

    impl crate::AuthorizationVerifier for AuthorizationVerifier {
        type Error = MockAuthorizationVerifierError;
        type Block = Block;
        type PublicKeyType = MockPublicKeyType;

        fn is_authorized(
            &self,
            at: &sp_api::BlockId<Block>,
            id: &MockPublicKeyType,
        ) -> Result<bool, MockAuthorizationVerifierError>;
    }

    impl Clone for AuthorizationVerifier {
        fn clone(&self) -> MockAuthorizationVerifier;
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

/// A helper function to get a current blockchain info.
fn prepare_get_info() -> sp_blockchain::Info<Block> {
    sp_blockchain::Info::<Block> {
        best_hash: sp_runtime::testing::H256::default(),
        best_number: 0,
        genesis_hash: sp_runtime::testing::H256::default(),
        finalized_hash: sp_runtime::testing::H256::default(),
        finalized_number: 0,
        finalized_state: None,
        number_leaves: 0,
    }
}

/// A helper function to prepare BlockImportParams.
fn prepare_block_import() -> BlockImportParams<Block, TransactionFor<MockClient, Block>> {
    BlockImportParams::new(
        BlockOrigin::Own,
        Header {
            parent_hash: Default::default(),
            number: 1,
            state_root: Default::default(),
            extrinsics_root: Default::default(),
            digest: Digest { logs: vec![] },
        },
    )
}

fn extract_bioauth_err(
    err: &sp_consensus::Error,
) -> &BioauthBlockImportError<MockBlockAuthorExtractorError, MockAuthorizationVerifierError> {
    if let sp_consensus::Error::Other(boxed_err) = err {
        if let Some(raw_err) =
            boxed_err.downcast_ref::<BioauthBlockImportError<
                MockBlockAuthorExtractorError,
                MockAuthorizationVerifierError,
            >>()
        {
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
async fn it_permits_block_import_with_valid_data() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

    let client = Arc::new(mock_client);

    let mut mock_block_author_extractor = MockBlockAuthorExtractor::new();
    mock_block_author_extractor
        .expect_extract_block_author()
        .returning(|_, _| Ok(()));

    let block_author_extractor = mock_block_author_extractor;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Ok(true));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        _,
        _,
        _,
        _,
    > = BioauthBlockImport::new(
        client,
        block_import,
        block_author_extractor,
        authorization_verifier,
    );

    let res = bioauth_block_import
        .import_block(prepare_block_import(), Default::default())
        .await;

    assert_eq!(res.unwrap(), ImportResult::imported(Default::default()));
}

#[tokio::test]
async fn it_denies_block_import_with_error_block_author_extractor() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

    let client = Arc::new(mock_client);

    let mut mock_block_author_extractor = MockBlockAuthorExtractor::new();
    mock_block_author_extractor
        .expect_extract_block_author()
        .returning(|_, _| Err(MockBlockAuthorExtractorError::BlockAuthorExtractorError));

    let block_author_extractor = mock_block_author_extractor;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Ok(true));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        _,
        _,
        _,
        _,
    > = BioauthBlockImport::new(
        client,
        block_import,
        block_author_extractor,
        authorization_verifier,
    );

    let res = bioauth_block_import
        .import_block(prepare_block_import(), Default::default())
        .await;

    assert_consensus_error!(
        res.unwrap_err(),
        BioauthBlockImportError::BlockAuthorExtraction(
            MockBlockAuthorExtractorError::BlockAuthorExtractorError
        )
    );
}

#[tokio::test]
async fn it_denies_block_import_with_error_authorization_verifier() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

    let client = Arc::new(mock_client);

    let mut mock_block_author_extractor = MockBlockAuthorExtractor::new();
    mock_block_author_extractor
        .expect_extract_block_author()
        .returning(|_, _| Ok(()));

    let block_author_extractor = mock_block_author_extractor;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Err(MockAuthorizationVerifierError::AuthorizationVerifierError));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        _,
        _,
        _,
        _,
    > = BioauthBlockImport::new(
        client,
        block_import,
        block_author_extractor,
        authorization_verifier,
    );

    let res = bioauth_block_import
        .import_block(prepare_block_import(), Default::default())
        .await;

    assert_consensus_error!(
        res.unwrap_err(),
        BioauthBlockImportError::AuthorizationVerifier(
            MockAuthorizationVerifierError::AuthorizationVerifierError
        )
    );
}

#[tokio::test]
async fn it_denies_block_import_with_error_not_bioauth_authorized() {
    let mut mock_client = MockClient::new();
    mock_client.expect_info().returning(prepare_get_info);

    let client = Arc::new(mock_client);

    let mut mock_block_author_extractor = MockBlockAuthorExtractor::new();
    mock_block_author_extractor
        .expect_extract_block_author()
        .returning(|_, _| Ok(()));

    let block_author_extractor = mock_block_author_extractor;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Ok(false));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_block_import = MockBlockImportWrapper::new();
    mock_block_import
        .expect_import_block()
        .returning(|_, _| Ok(ImportResult::Imported(Default::default())));

    let block_import = mock_block_import;

    let mut bioauth_block_import: BioauthBlockImport<
        sc_service::TFullBackend<Block>,
        _,
        _,
        _,
        _,
        _,
    > = BioauthBlockImport::new(
        client,
        block_import,
        block_author_extractor,
        authorization_verifier,
    );

    let res = bioauth_block_import
        .import_block(prepare_block_import(), Default::default())
        .await;

    assert_consensus_error!(
        res.unwrap_err(),
        BioauthBlockImportError::NotBioauthAuthorized
    );
}
