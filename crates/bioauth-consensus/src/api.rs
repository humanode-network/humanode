//! API integration.

use bioauth_consensus_api::BioauthConsensusApi;
use bioauth_id_api::BioauthIdApi;
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;
use std::{marker::PhantomData, sync::Arc};

/// Provides an authorization verifier on top of bioauth consensus API.
#[derive(Debug)]
pub struct AuthorizationVerifier<Block: BlockT, Client, ValidatorId, BioauthId> {
    /// The client provides access to the runtime.
    client: Arc<Client>,
    /// The type of the block used in the chain.
    _phantom_block: PhantomData<Block>,
    /// The type of the validator indentity used in the chain.
    _phantom_validator_id: PhantomData<ValidatorId>,
    /// The type of the bioauth indentity used in the chain.
    _phantom_bioauth_id: PhantomData<BioauthId>,
}

/// An error that can occur during authorization verification.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationVerifierError {
    /// Something went wrong while extracting the authorized ids from the chain state via
    /// the runtime.
    #[error("unable to extract authorized ids from the chain state: {0}")]
    UnableToExtractAuthorizedIds(sp_api::ApiError),
    /// Something went wrong while extracting the corresponding BioauthId based on
    /// provided ValidatorId.
    #[error("unable to extract bioauth id based on provided validator id: {0}")]
    UnableToExtractBioauthId(sp_api::ApiError),
    /// BioauthId has't been found based on provided validator id.
    #[error("unable to obtaion the slot from the block header")]
    BioauthIdNotFound,
}

impl<Block: BlockT, Client, ValidatorId, BioauthId>
    AuthorizationVerifier<Block, Client, ValidatorId, BioauthId>
{
    /// Create a new [`AuthorizationVerifier`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_block: PhantomData,
            _phantom_validator_id: PhantomData,
            _phantom_bioauth_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, ValidatorId, BioauthId> Clone
    for AuthorizationVerifier<Block, Client, ValidatorId, BioauthId>
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_block: PhantomData,
            _phantom_validator_id: PhantomData,
            _phantom_bioauth_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, ValidatorId, BioauthId> crate::AuthorizationVerifier
    for AuthorizationVerifier<Block, Client, ValidatorId, BioauthId>
where
    Client: ProvideRuntimeApi<Block>,
    Client::Api: BioauthConsensusApi<Block, BioauthId>,
    Client::Api: BioauthIdApi<Block, ValidatorId, BioauthId>,
    BioauthId: codec::Encode + codec::Decode + PartialEq,
    ValidatorId: codec::Encode,
{
    type Error = AuthorizationVerifierError;
    type Block = Block;
    type PublicKeyType = ValidatorId;

    fn is_authorized(
        &self,
        at: &BlockId<Self::Block>,
        validator_id: &Self::PublicKeyType,
    ) -> Result<bool, Self::Error> {
        let bioauth_id = self
            .client
            .runtime_api()
            .extract_bioauth_id(at, validator_id)
            .map_err(AuthorizationVerifierError::UnableToExtractBioauthId)?
            .ok_or(AuthorizationVerifierError::BioauthIdNotFound)?;

        let is_authorized = self
            .client
            .runtime_api()
            .is_authorized(at, &bioauth_id)
            .map_err(AuthorizationVerifierError::UnableToExtractAuthorizedIds)?;
        Ok(is_authorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use node_primitives::Block;
    use sp_api::{ApiError, ApiRef, NativeOrEncoded, ProvideRuntimeApi};
    use std::sync::Arc;

    type MockPublicKeyType = ();
    type MockBioauthId = ();

    mock! {
        RuntimeApi {
            fn is_authorized(&self, at: &sp_api::BlockId<Block>, id: &MockBioauthId) -> Result<NativeOrEncoded<bool>, ApiError>;
            fn extract_bioauth_id(&self, at: &sp_api::BlockId<Block>, id: &MockPublicKeyType) -> Result<NativeOrEncoded<MockBioauthId>, ApiError>;
        }
    }

    #[derive(Clone)]
    struct MockWrapperRuntimeApi(Arc<MockRuntimeApi>);

    sp_api::mock_impl_runtime_apis! {
        impl bioauth_consensus_api::BioauthConsensusApi<Block, MockPublicKeyType> for MockWrapperRuntimeApi {

            #[advanced]
            fn is_authorized(&self, at: &sp_api::BlockId<Block>, id: &MockBioauthId) -> Result<NativeOrEncoded<bool>, ApiError> {
                self.0.is_authorized(at, id)
            }
        }

        impl bioauth_id_api::BioauthIdApi<Block, MockPublicKeyType, MockBioauthId> for MockWrapperRuntimeApi {
            #[advanced]
            fn extract_bioauth_id(&self, at: &sp_api::BlockId<Block>, id: &MockPublicKeyType) -> Result<NativeOrEncoded<MockBioauthId>, ApiError> {
                self.0.extract_bioauth_id(at, id)
            }
        }
    }

    mock! {
        #[derive(Debug)]
        Client {}

        impl ProvideRuntimeApi<Block> for Client {
            type Api = MockWrapperRuntimeApi;

            fn runtime_api<'a>(&'a self) -> ApiRef<'a, MockWrapperRuntimeApi>;
        }
    }

    /// This test verifies authorization success when a respective runtime_api calls (extract_bioauth_id, is_authorized)
    /// succeed and the provided id is authorized.
    #[test]
    fn success() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_extract_bioauth_id()
            .returning(|_, _| Ok(NativeOrEncoded::from(())));
        mock_runtime_api
            .expect_is_authorized()
            .returning(|_, _| Ok(NativeOrEncoded::from(true)));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let authorization_verifier: AuthorizationVerifier<
            Block,
            _,
            MockPublicKeyType,
            MockBioauthId,
        > = AuthorizationVerifier::new(Arc::clone(&client));

        let res = crate::AuthorizationVerifier::is_authorized(
            &authorization_verifier,
            &sp_api::BlockId::Number(0),
            &MockPublicKeyType::default(),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(authorization_verifier);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        assert!(res.unwrap());
    }

    /// This test verifies authorization failure when a respective runtime_api calls (extract_bioauth_id, is_authorized)
    /// succeed, but the provided id isn't authorized.
    #[test]
    fn error_id_not_bioauth_authorized() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_extract_bioauth_id()
            .returning(|_, _| Ok(NativeOrEncoded::from(())));
        mock_runtime_api
            .expect_is_authorized()
            .returning(|_, _| Ok(NativeOrEncoded::from(false)));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let authorization_verifier: AuthorizationVerifier<
            Block,
            _,
            MockPublicKeyType,
            MockBioauthId,
        > = AuthorizationVerifier::new(Arc::clone(&client));

        let res = crate::AuthorizationVerifier::is_authorized(
            &authorization_verifier,
            &sp_api::BlockId::Number(0),
            &MockPublicKeyType::default(),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(authorization_verifier);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        assert!(!res.unwrap());
    }

    /// This test verifies authorization failure when a respective runtime_api call (is_authorized) fails.
    #[test]
    fn runtime_is_authorized_error() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_extract_bioauth_id()
            .returning(|_, _| Ok(NativeOrEncoded::from(())));
        mock_runtime_api.expect_is_authorized().returning(|_, _| {
            Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
        });

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let authorization_verifier: AuthorizationVerifier<
            Block,
            _,
            MockPublicKeyType,
            MockBioauthId,
        > = AuthorizationVerifier::new(Arc::clone(&client));

        let res = crate::AuthorizationVerifier::is_authorized(
            &authorization_verifier,
            &sp_api::BlockId::Number(0),
            &MockPublicKeyType::default(),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(authorization_verifier);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        match res.unwrap_err() {
            AuthorizationVerifierError::UnableToExtractAuthorizedIds(e)
                if e.to_string() == "Test error" => {}
            ref e => panic!(
                "assertion failed: `{:?}` does not match `{}`",
                e,
                stringify!(AuthorizationVerifierError::UnableToExtractAuthorizedIds(
                    "Test error"
                ))
            ),
        }
    }

    /// This test verifies authorization failure when a respective runtime_api call (extract_bioauth_id) fails.
    #[test]
    fn runtime_extract_bioauth_id_error() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_extract_bioauth_id()
            .returning(|_, _| {
                Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
            });

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let authorization_verifier: AuthorizationVerifier<
            Block,
            _,
            MockPublicKeyType,
            MockBioauthId,
        > = AuthorizationVerifier::new(Arc::clone(&client));

        let res = crate::AuthorizationVerifier::is_authorized(
            &authorization_verifier,
            &sp_api::BlockId::Number(0),
            &MockPublicKeyType::default(),
        );

        // Drop the test object and all the mocks in it, effectively running the mock assertions.
        drop(authorization_verifier);
        // Unwrap the client from the Arc and drop it, ensuring it's mock assertions run too.
        drop(Arc::try_unwrap(client).unwrap());

        match res.unwrap_err() {
            AuthorizationVerifierError::UnableToExtractBioauthId(e)
                if e.to_string() == "Test error" => {}
            ref e => panic!(
                "assertion failed: `{:?}` does not match `{}`",
                e,
                stringify!(AuthorizationVerifierError::UnableToExtractBioauthId(
                    "Test error"
                ))
            ),
        }
    }
}
