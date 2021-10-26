use futures::future;
use mockall::predicate::*;
use mockall::*;
use node_primitives::{Block, BlockNumber, Hash, Header};
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{ApiRef, ProvideRuntimeApi, TransactionFor};
use sp_blockchain::{well_known_cache_keys, HeaderBackend};
use sp_consensus::{BlockOrigin, Environment, Error as ConsensusError};
use sp_runtime::{
    traits::{Block as BlockT, DigestFor},
    Digest,
};
use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{BioauthBlockImport, BioauthBlockImportError, BioauthProposer, BioauthProposerError};

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
    #[error("")]
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
    #[error("")]
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

/// BioauthProposer requires impl std::error::Error.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum MockValidatorKeyExtractorError {
    #[error("")]
    ValidatorKeyExtractorError,
}

mock! {
    ValidatorKeyExtractor {}

    impl crate::ValidatorKeyExtractor for ValidatorKeyExtractor {
        type Error = MockValidatorKeyExtractorError;
        type PublicKeyType = MockPublicKeyType;

        fn extract_validator_key(&self) -> Result<Option<MockPublicKeyType>, MockValidatorKeyExtractorError>;
    }
}

type MockProposal = future::Ready<
    Result<
        sp_consensus::Proposal<Block, TransactionFor<MockClient, Block>, ()>,
        sp_blockchain::Error,
    >,
>;

mock! {
    Proposer {
        fn propose(
            &self,
            inherent_data: sp_inherents::InherentData,
            inherent_digests: DigestFor<Block>,
            max_duration: Duration,
            block_size_limit: Option<usize>,
        ) -> MockProposal;
    }
}

#[derive(Clone)]
struct MockWrapperProposer(Arc<MockProposer>, &'static str);

/// We need to be able to compare MockProposer.
impl std::cmp::PartialEq for MockWrapperProposer {
    fn eq(&self, other: &MockWrapperProposer) -> bool {
        if self.1 == other.1 {
            return true;
        }
        false
    }
}

/// We need to be able to debug MockProposer.
impl std::fmt::Debug for MockWrapperProposer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}

impl sp_consensus::Proposer<Block> for MockWrapperProposer {
    type Error = sp_blockchain::Error;
    type Transaction = TransactionFor<MockClient, Block>;
    type Proposal = MockProposal;
    type ProofRecording = sp_consensus::DisableProofRecording;
    type Proof = ();

    fn propose(
        self,
        inherent_data: sp_inherents::InherentData,
        inherent_digests: DigestFor<Block>,
        max_duration: Duration,
        block_size_limit: Option<usize>,
    ) -> MockProposal {
        self.0.propose(
            inherent_data,
            inherent_digests,
            max_duration,
            block_size_limit,
        )
    }
}

mock! {
    BasicAuthorshipProposer {}

    impl Environment<Block> for BasicAuthorshipProposer {
        type Proposer = MockWrapperProposer;
        type CreateProposer = future::BoxFuture<'static, Result<MockWrapperProposer, sp_blockchain::Error>>;
        type Error = sp_blockchain::Error;

        fn init(&mut self, parent_header: &<Block as BlockT>::Header) -> future::BoxFuture<'static, Result<MockWrapperProposer, sp_blockchain::Error>>;
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

fn extract_proposer_err(
    err: &sp_blockchain::Error,
) -> &BioauthProposerError<MockValidatorKeyExtractorError, MockAuthorizationVerifierError> {
    if let sp_blockchain::Error::Consensus(sp_consensus::Error::Other(boxed_err)) = err {
        if let Some(raw_err) =
            boxed_err.downcast_ref::<BioauthProposerError<
                MockValidatorKeyExtractorError,
                MockAuthorizationVerifierError,
            >>()
        {
            return raw_err;
        }
    }
    panic!("Unexpected proposer error: {}", err);
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

#[tokio::test]
async fn it_permits_bioauth_proposer() {
    let mock_proposer = MockProposer::new();
    let wrapper_proposer = MockWrapperProposer(Arc::new(mock_proposer), "Test proposer");
    let cloned_wrapper_proposer = wrapper_proposer.clone();

    let mut mock_basic_authorship_proposer = MockBasicAuthorshipProposer::new();
    mock_basic_authorship_proposer
        .expect_init()
        .returning(move |_| Box::pin(future::ready(Ok(cloned_wrapper_proposer.clone()))));

    let basic_authorship_proposer = mock_basic_authorship_proposer;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Ok(true));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_validator_key_extractor = MockValidatorKeyExtractor::new();
    mock_validator_key_extractor
        .expect_extract_validator_key()
        .returning(|| Ok(Some(())));

    let validator_key_extractor = mock_validator_key_extractor;

    let mut bioauth_proposer: BioauthProposer<Block, _, _, _> = BioauthProposer::new(
        basic_authorship_proposer,
        validator_key_extractor,
        authorization_verifier,
    );

    let res = bioauth_proposer.init(&Header {
        parent_hash: Default::default(),
        number: 1,
        state_root: Default::default(),
        extrinsics_root: Default::default(),
        digest: Digest { logs: vec![] },
    });

    assert_eq!(res.await.unwrap(), wrapper_proposer);
}

#[tokio::test]
async fn it_denies_bioauth_proposer_with_error_validator_key_extractor() {
    let mock_proposer = MockProposer::new();
    let wrapper_proposer = MockWrapperProposer(Arc::new(mock_proposer), "Test proposer");
    let cloned_wrapper_proposer = wrapper_proposer.clone();

    let mut mock_basic_authorship_proposer = MockBasicAuthorshipProposer::new();
    mock_basic_authorship_proposer
        .expect_init()
        .returning(move |_| Box::pin(future::ready(Ok(cloned_wrapper_proposer.clone()))));

    let basic_authorship_proposer = mock_basic_authorship_proposer;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Ok(true));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_validator_key_extractor = MockValidatorKeyExtractor::new();
    mock_validator_key_extractor
        .expect_extract_validator_key()
        .returning(|| Err(MockValidatorKeyExtractorError::ValidatorKeyExtractorError));

    let validator_key_extractor = mock_validator_key_extractor;

    let mut bioauth_proposer: BioauthProposer<Block, _, _, _> = BioauthProposer::new(
        basic_authorship_proposer,
        validator_key_extractor,
        authorization_verifier,
    );

    let res = bioauth_proposer.init(&Header {
        parent_hash: Default::default(),
        number: 1,
        state_root: Default::default(),
        extrinsics_root: Default::default(),
        digest: Digest { logs: vec![] },
    });

    let err = res.await.unwrap_err();
    let err = extract_proposer_err(&err);

    match err {
        BioauthProposerError::ValidatorKeyExtraction(
            MockValidatorKeyExtractorError::ValidatorKeyExtractorError,
        ) => (),
        ref e => panic!(
            "assertion failed: `{:?}` does not match `{}`",
            e,
            BioauthProposerError::ValidatorKeyExtraction::<
                MockValidatorKeyExtractorError,
                MockAuthorizationVerifierError,
            >(MockValidatorKeyExtractorError::ValidatorKeyExtractorError,)
        ),
    }
}

#[tokio::test]
async fn it_denies_bioauth_proposer_with_error_unable_to_extract_validator_key() {
    let mock_proposer = MockProposer::new();
    let wrapper_proposer = MockWrapperProposer(Arc::new(mock_proposer), "Test proposer");
    let cloned_wrapper_proposer = wrapper_proposer.clone();

    let mut mock_basic_authorship_proposer = MockBasicAuthorshipProposer::new();
    mock_basic_authorship_proposer
        .expect_init()
        .returning(move |_| Box::pin(future::ready(Ok(cloned_wrapper_proposer.clone()))));

    let basic_authorship_proposer = mock_basic_authorship_proposer;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Ok(true));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_validator_key_extractor = MockValidatorKeyExtractor::new();
    mock_validator_key_extractor
        .expect_extract_validator_key()
        .returning(|| Ok(None));

    let validator_key_extractor = mock_validator_key_extractor;

    let mut bioauth_proposer: BioauthProposer<Block, _, _, _> = BioauthProposer::new(
        basic_authorship_proposer,
        validator_key_extractor,
        authorization_verifier,
    );

    let res = bioauth_proposer.init(&Header {
        parent_hash: Default::default(),
        number: 1,
        state_root: Default::default(),
        extrinsics_root: Default::default(),
        digest: Digest { logs: vec![] },
    });

    let err = res.await.unwrap_err();
    let err = extract_proposer_err(&err);

    match err {
        BioauthProposerError::UnableToExtractValidatorKey => (),
        ref e => panic!(
            "assertion failed: `{:?}` does not match `{}`",
            e,
            BioauthProposerError::UnableToExtractValidatorKey::<
                MockValidatorKeyExtractorError,
                MockAuthorizationVerifierError,
            >
        ),
    }
}

#[tokio::test]
async fn it_denies_bioauth_proposer_with_error_authorization_verification() {
    let mock_proposer = MockProposer::new();
    let wrapper_proposer = MockWrapperProposer(Arc::new(mock_proposer), "Test proposer");
    let cloned_wrapper_proposer = wrapper_proposer.clone();

    let mut mock_basic_authorship_proposer = MockBasicAuthorshipProposer::new();
    mock_basic_authorship_proposer
        .expect_init()
        .returning(move |_| Box::pin(future::ready(Ok(cloned_wrapper_proposer.clone()))));

    let basic_authorship_proposer = mock_basic_authorship_proposer;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Err(MockAuthorizationVerifierError::AuthorizationVerifierError));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_validator_key_extractor = MockValidatorKeyExtractor::new();
    mock_validator_key_extractor
        .expect_extract_validator_key()
        .returning(|| Ok(Some(())));

    let validator_key_extractor = mock_validator_key_extractor;

    let mut bioauth_proposer: BioauthProposer<Block, _, _, _> = BioauthProposer::new(
        basic_authorship_proposer,
        validator_key_extractor,
        authorization_verifier,
    );

    let res = bioauth_proposer.init(&Header {
        parent_hash: Default::default(),
        number: 1,
        state_root: Default::default(),
        extrinsics_root: Default::default(),
        digest: Digest { logs: vec![] },
    });

    let err = res.await.unwrap_err();
    let err = extract_proposer_err(&err);

    match err {
        BioauthProposerError::AuthorizationVerification(
            MockAuthorizationVerifierError::AuthorizationVerifierError,
        ) => (),
        ref e => panic!(
            "assertion failed: `{:?}` does not match `{}`",
            e,
            BioauthProposerError::AuthorizationVerification::<
                MockValidatorKeyExtractorError,
                MockAuthorizationVerifierError,
            >(MockAuthorizationVerifierError::AuthorizationVerifierError,)
        ),
    }
}

#[tokio::test]
async fn it_denies_bioauth_proposer_with_error_not_bioauth_authorized() {
    let mock_proposer = MockProposer::new();
    let wrapper_proposer = MockWrapperProposer(Arc::new(mock_proposer), "Test proposer");
    let cloned_wrapper_proposer = wrapper_proposer.clone();

    let mut mock_basic_authorship_proposer = MockBasicAuthorshipProposer::new();
    mock_basic_authorship_proposer
        .expect_init()
        .returning(move |_| Box::pin(future::ready(Ok(cloned_wrapper_proposer.clone()))));

    let basic_authorship_proposer = mock_basic_authorship_proposer;

    let mut mock_authorization_verifier = MockAuthorizationVerifier::new();
    mock_authorization_verifier
        .expect_is_authorized()
        .returning(|_, _| Ok(false));

    let authorization_verifier = mock_authorization_verifier;

    let mut mock_validator_key_extractor = MockValidatorKeyExtractor::new();
    mock_validator_key_extractor
        .expect_extract_validator_key()
        .returning(|| Ok(Some(())));

    let validator_key_extractor = mock_validator_key_extractor;

    let mut bioauth_proposer: BioauthProposer<Block, _, _, _> = BioauthProposer::new(
        basic_authorship_proposer,
        validator_key_extractor,
        authorization_verifier,
    );

    let res = bioauth_proposer.init(&Header {
        parent_hash: Default::default(),
        number: 1,
        state_root: Default::default(),
        extrinsics_root: Default::default(),
        digest: Digest { logs: vec![] },
    });

    let err = res.await.unwrap_err();
    let err = extract_proposer_err(&err);

    match err {
        BioauthProposerError::NotBioauthAuthorized => (),
        ref e => panic!(
            "assertion failed: `{:?}` does not match `{}`",
            e,
            BioauthProposerError::NotBioauthAuthorized::<
                MockValidatorKeyExtractorError,
                MockAuthorizationVerifierError,
            >
        ),
    }
}
