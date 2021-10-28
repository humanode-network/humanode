//! API integration.

use bioauth_consensus_api::BioauthConsensusApi;
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;
use std::{marker::PhantomData, sync::Arc};

/// Provides an authorization verifier on top of bioauth consensus API.
#[derive(Debug)]
pub struct AuthorizationVerifier<Block: BlockT, Client, Id> {
    /// The client provides access to the runtime.
    client: Arc<Client>,
    /// The type of the block used in the chain.
    _phantom_block: PhantomData<Block>,
    /// The type of the indentity used in the chain.
    _phantom_id: PhantomData<Id>,
}

/// An error that can occur during authorization verification.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationVerifierError {
    /// Something went wrong while extracting the authorized ids from the chain state via
    /// the runtime.
    #[error("unable to extract authorized ids from the chain state: {0}")]
    UnableToExtractAuthorizedIds(sp_api::ApiError),
}

impl<Block: BlockT, Client, Id> AuthorizationVerifier<Block, Client, Id> {
    /// Create a new [`AuthorizationVerifier`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_block: PhantomData,
            _phantom_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, Id> Clone for AuthorizationVerifier<Block, Client, Id> {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_block: PhantomData,
            _phantom_id: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, Id> crate::AuthorizationVerifier
    for AuthorizationVerifier<Block, Client, Id>
where
    Client: ProvideRuntimeApi<Block>,
    Client::Api: BioauthConsensusApi<Block, Id>,
    Id: codec::Encode + PartialEq,
{
    type Error = AuthorizationVerifierError;
    type Block = Block;
    type PublicKeyType = Id;

    fn is_authorized(
        &self,
        at: &BlockId<Self::Block>,
        id: &Self::PublicKeyType,
    ) -> Result<bool, Self::Error> {
        let is_authorized = self
            .client
            .runtime_api()
            .is_authorized(at, id)
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

    mock! {
        RuntimeApi {
            fn is_authorized(&self, at: &sp_api::BlockId<Block>, id: &MockPublicKeyType) -> Result<NativeOrEncoded<bool>, ApiError>;
        }
    }

    #[derive(Clone)]
    struct MockWrapperRuntimeApi(Arc<MockRuntimeApi>);

    sp_api::mock_impl_runtime_apis! {
        impl bioauth_consensus_api::BioauthConsensusApi<Block, MockPublicKeyType> for MockWrapperRuntimeApi {

            #[advanced]
            fn is_authorized(&self, at: &sp_api::BlockId<Block>, id: &MockPublicKeyType) -> Result<NativeOrEncoded<bool>, ApiError> {
                self.0.is_authorized(at, id)
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

    /// This test verifies authorizatin success if a respective runtime_api call (is_authorized)
    /// succeeds and the provided id is already authorized.
    #[test]
    fn success() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_is_authorized()
            .returning(|_, _| Ok(NativeOrEncoded::from(true)));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let authorization_verifier: AuthorizationVerifier<Block, _, MockPublicKeyType> =
            AuthorizationVerifier::new(Arc::clone(&client));

        let res = crate::AuthorizationVerifier::is_authorized(
            &authorization_verifier,
            &sp_api::BlockId::Number(0),
            &MockPublicKeyType::default(),
        );

        assert!(res.unwrap());
    }

    /// This test verifies authorizatin failer if a respective runtime_api call (is_authorized)
    /// succeeds, but the provided id isn't authorized.
    #[test]
    fn error_id_not_bioauth_authorized() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api
            .expect_is_authorized()
            .returning(|_, _| Ok(NativeOrEncoded::from(false)));

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let authorization_verifier: AuthorizationVerifier<Block, _, MockPublicKeyType> =
            AuthorizationVerifier::new(Arc::clone(&client));

        let res = crate::AuthorizationVerifier::is_authorized(
            &authorization_verifier,
            &sp_api::BlockId::Number(0),
            &MockPublicKeyType::default(),
        );

        assert!(!res.unwrap());
    }

    /// This test verifies authorizatin failer if a respective runtime_api call (is_authorized) fails.
    #[test]
    fn runtime_error() {
        let mut mock_client = MockClient::new();

        let mut mock_runtime_api = MockRuntimeApi::new();
        mock_runtime_api.expect_is_authorized().returning(|_, _| {
            Err((Box::from("Test error") as Box<dyn std::error::Error + Send + Sync>).into())
        });

        let runtime_api = MockWrapperRuntimeApi(Arc::new(mock_runtime_api));

        mock_client
            .expect_runtime_api()
            .returning(move || runtime_api.clone().into());

        let client = Arc::new(mock_client);

        let authorization_verifier: AuthorizationVerifier<Block, _, MockPublicKeyType> =
            AuthorizationVerifier::new(Arc::clone(&client));

        let res = crate::AuthorizationVerifier::is_authorized(
            &authorization_verifier,
            &sp_api::BlockId::Number(0),
            &MockPublicKeyType::default(),
        );

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
}
