use futures::future;
use node_primitives::{Block, Header};
use sc_consensus::{BlockImport, BlockImportParams, ImportResult};
use sp_api::TransactionFor;
use sp_consensus::{BlockOrigin, Environment};
use sp_runtime::Digest;
use std::sync::Arc;

use crate::mock::*;
use crate::{BioauthBlockImport, BioauthBlockImportError, BioauthProposer, BioauthProposerError};

type ImportError =
    BioauthBlockImportError<MockBlockAuthorExtractorError, MockAuthorizationVerifierError>;
type ProposerError =
    BioauthProposerError<MockValidatorKeyExtractorError, MockAuthorizationVerifierError>;

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

fn extract_bioauth_err<ErrorKind: 'static + std::error::Error>(
    err: &sp_consensus::Error,
) -> &ErrorKind {
    if let sp_consensus::Error::Other(boxed_err) = err {
        if let Some(raw_err) = boxed_err.downcast_ref::<ErrorKind>() {
            return raw_err;
        }
    }
    panic!("Unexpected consensus error: {}", err);
}

macro_rules! assert_consensus_error {
    ($expression:expr, $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?, $typ:ty) => {
        {
            let err_hold = $expression;
            let err = extract_bioauth_err::<$typ>(&err_hold);
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

/// This test verifies block import success when a block author is extracted succesfully
/// and an authorization verifier succeeds where the block author is authorized.
#[tokio::test]
async fn block_import_success() {
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

/// This test verifies block import failure when a block author extractor fails.
#[tokio::test]
async fn block_import_error_block_author_extraction() {
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
        ),
        ImportError
    );
}

/// This test verifies block import failure when an authorization verifier fails.
#[tokio::test]
async fn block_import_error_authorization_verifier() {
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
        ),
        ImportError
    );
}

/// This test verifies block import failer when a block author isn't authorized.
#[tokio::test]
async fn block_import_error_not_bioauth_authorized() {
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
        BioauthBlockImportError::NotBioauthAuthorized,
        ImportError
    );
}

/// This test verifies proposer success when a validator key is extracted succesfully
/// and an authorization verifier succeeds where the owner of the validator key is authorized.
#[tokio::test]
async fn proposer_success() {
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

/// This test verifies proposer failure when a validator key extractor fails.
#[tokio::test]
async fn proposer_error_validator_key_extractor() {
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

    assert_consensus_error!(
        res.await.unwrap_err(),
        BioauthProposerError::ValidatorKeyExtraction(
            MockValidatorKeyExtractorError::ValidatorKeyExtractorError,
        ),
        ProposerError
    );
}

/// This test verifies proposer failure when a validator key extractor succeeds
/// but the key cann't be extracted as the validator key.
#[tokio::test]
async fn proposer_error_unable_to_extract_validator_key() {
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

    assert_consensus_error!(
        res.await.unwrap_err(),
        BioauthProposerError::UnableToExtractValidatorKey,
        ProposerError
    );
}

/// This test verifies proposer failure when an authorization verifier fails.
#[tokio::test]
async fn proposer_error_authorization_verifier() {
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

    assert_consensus_error!(
        res.await.unwrap_err(),
        BioauthProposerError::AuthorizationVerification(
            MockAuthorizationVerifierError::AuthorizationVerifierError,
        ),
        ProposerError
    );
}

/// This test verifies proposer failure when the owner of the validator key isn't authorized.
#[tokio::test]
async fn proposer_error_not_bioauth_authorized() {
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

    assert_consensus_error!(
        res.await.unwrap_err(),
        BioauthProposerError::NotBioauthAuthorized,
        ProposerError
    );
}
