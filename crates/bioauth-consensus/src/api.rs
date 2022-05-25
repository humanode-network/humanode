//! API integration.

use std::{marker::PhantomData, sync::Arc};

use bioauth_consensus_api::{BioauthConsensusApi, BioauthConsensusSessionApi};
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;

/// The abstract interface to the runtime, allowing various invocation methods to the logic to check
/// the authorization state.
pub trait RuntimeApiChecker<Block, Client, Id>
where
    Block: BlockT,
    Client: ProvideRuntimeApi<Block>,
    Id: codec::Encode,
{
    /// The call that returns whether the [`Id`] is authorized on not.
    fn is_authorized(
        client: &Client,
        at: &BlockId<Block>,
        id: &Id,
    ) -> Result<bool, sp_api::ApiError>;
}

/// [`Direct`] performs the check via the [`BioauthConsensusApi::is_authorized`] call.
pub struct Direct;

impl<Block, Client, Id> RuntimeApiChecker<Block, Client, Id> for Direct
where
    Block: BlockT,
    Client: ProvideRuntimeApi<Block>,
    Client::Api: BioauthConsensusApi<Block, Id>,
    Id: codec::Encode,
{
    fn is_authorized(
        client: &Client,
        at: &BlockId<Block>,
        id: &Id,
    ) -> Result<bool, sp_api::ApiError> {
        client.runtime_api().is_authorized(at, id)
    }
}

/// [`Session`] performs the check via
/// the [`BioauthConsensusSessionApi::is_authorized_through_session_key`] call.
pub struct Session;

impl<Block, Client, Id> RuntimeApiChecker<Block, Client, Id> for Session
where
    Block: BlockT,
    Client: ProvideRuntimeApi<Block>,
    Client::Api: BioauthConsensusSessionApi<Block, Id>,
    Id: codec::Encode,
{
    fn is_authorized(
        client: &Client,
        at: &BlockId<Block>,
        id: &Id,
    ) -> Result<bool, sp_api::ApiError> {
        client
            .runtime_api()
            .is_authorized_through_session_key(at, id)
    }
}

/// Provides an authorization verifier on top of bioauth consensus API.
#[derive(Debug)]
pub struct AuthorizationVerifier<Block: BlockT, Client, Id, Checker> {
    /// The client provides access to the runtime.
    client: Arc<Client>,
    /// The phantom types.
    _phantom_types: PhantomData<(Block, Id, Checker)>,
}

/// An error that can occur during authorization verification.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationVerifierError {
    /// Something went wrong while extracting the authorization status from the runtime.
    #[error("unable to check authorization: {0}")]
    UnableToCheckAuthorization(sp_api::ApiError),
}

impl<Block: BlockT, Client, Id, Checker> AuthorizationVerifier<Block, Client, Id, Checker> {
    /// Create a new [`AuthorizationVerifier`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_types: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, Id, Checker> Clone
    for AuthorizationVerifier<Block, Client, Id, Checker>
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_types: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, Id, Checker> crate::AuthorizationVerifier
    for AuthorizationVerifier<Block, Client, Id, Checker>
where
    Client: ProvideRuntimeApi<Block>,
    Id: codec::Encode,
    Checker: RuntimeApiChecker<Block, Client, Id>,
{
    type Error = AuthorizationVerifierError;
    type Block = Block;
    type PublicKeyType = Id;

    fn is_authorized(
        &self,
        at: &BlockId<Self::Block>,
        id: &Self::PublicKeyType,
    ) -> Result<bool, Self::Error> {
        let is_authorized = Checker::is_authorized(self.client.as_ref(), at, id)
            .map_err(AuthorizationVerifierError::UnableToCheckAuthorization)?;
        Ok(is_authorized)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::*;
    use mockall::*;
    use node_primitives::Block;
    use sp_api::{ApiError, ApiRef, NativeOrEncoded, ProvideRuntimeApi};

    use super::*;

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

    /// This test verifies authorization success when a respective runtime_api call (is_authorized)
    /// succeeds and the provided id is authorized.
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

        let authorization_verifier: AuthorizationVerifier<Block, _, MockPublicKeyType, Direct> =
            AuthorizationVerifier::new(Arc::clone(&client));

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

    /// This test verifies authorization failure when a respective runtime_api call (is_authorized)
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

        let authorization_verifier: AuthorizationVerifier<Block, _, MockPublicKeyType, Direct> =
            AuthorizationVerifier::new(Arc::clone(&client));

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

        let authorization_verifier: AuthorizationVerifier<Block, _, MockPublicKeyType, Direct> =
            AuthorizationVerifier::new(Arc::clone(&client));

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
            AuthorizationVerifierError::UnableToCheckAuthorization(e)
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
