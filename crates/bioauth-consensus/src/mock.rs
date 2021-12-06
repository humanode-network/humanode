use futures::future;
use mockall::predicate::*;
use mockall::*;
use node_primitives::{Block, BlockNumber, Hash, Header};
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{ApiRef, ProvideRuntimeApi, TransactionFor};
use sp_blockchain::{well_known_cache_keys, HeaderBackend};
use sp_consensus::{Environment, Error as ConsensusError};
use sp_runtime::{traits::Block as BlockT, Digest};
use std::{collections::HashMap, sync::Arc, time::Duration};

type MockPublicKeyType = ();
type MockRuntimeApi = ();

#[derive(Clone)]
pub struct MockWrapperRuntimeApi(Arc<MockRuntimeApi>);

/// The Fake trait is used to reuse required implementations from mock_impl_runtime_apis
/// macro that should be used in impl ProvideRuntimeApi.
trait MockFakeTrait<Block> {}

sp_api::mock_impl_runtime_apis! {
    impl MockFakeTrait<Block> for MockWrapperRuntimeApi {}
}

mock! {
    #[derive(Debug)]
    pub Client {}

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
    pub BlockAuthorExtractor {}

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
    pub AuthorizationVerifier {}

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
    pub BlockImportWrapper {}

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
    pub ValidatorKeyExtractor {}

    impl crate::ValidatorKeyExtractor for ValidatorKeyExtractor {
        type Error = MockValidatorKeyExtractorError;
        type PublicKeyType = MockPublicKeyType;

        fn extract_validator_key(&self) -> Result<Option<MockPublicKeyType>, MockValidatorKeyExtractorError>;
    }
}

type MockProposal = future::Ready<
    Result<
        sp_consensus::Proposal<Block, TransactionFor<MockClient, Block>, ()>,
        sp_consensus::Error,
    >,
>;

mock! {
    pub Proposer {
        fn propose(
            &self,
            inherent_data: sp_inherents::InherentData,
            inherent_digests: Digest,
            max_duration: Duration,
            block_size_limit: Option<usize>,
        ) -> MockProposal;
    }
}

#[derive(Clone)]
pub struct MockWrapperProposer(pub Arc<MockProposer>, pub &'static str);

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
    type Error = sp_consensus::Error;
    type Transaction = TransactionFor<MockClient, Block>;
    type Proposal = MockProposal;
    type ProofRecording = sp_consensus::DisableProofRecording;
    type Proof = ();

    fn propose(
        self,
        inherent_data: sp_inherents::InherentData,
        inherent_digests: Digest,
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
    pub BasicAuthorshipProposer {}

    impl Environment<Block> for BasicAuthorshipProposer {
        type Proposer = MockWrapperProposer;
        type CreateProposer = future::BoxFuture<'static, Result<MockWrapperProposer, sp_consensus::Error>>;
        type Error = sp_consensus::Error;

        fn init(&mut self, parent_header: &<Block as BlockT>::Header) -> future::BoxFuture<'static, Result<MockWrapperProposer, sp_consensus::Error>>;
    }
}
