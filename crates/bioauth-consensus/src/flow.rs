//! Flow integration.

use std::{marker::PhantomData, sync::Arc};

use bioauth_flow_api::BioauthFlowApi;
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_runtime::traits::Block as BlockT;

/// Provides an authorization verifier on top of bioauth flow API.
#[derive(Debug)]
pub struct AuthorizationVerifier<Block: BlockT, Client, Id, Timestamp> {
    /// The client provides access to the runtime.
    client: Arc<Client>,
    /// The phantiom types.
    _phantom_types: PhantomData<(Block, Id, Timestamp)>,
}

/// An error that can occur during authorization verification.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationVerifierError {
    /// Something went wrong while extracting the bioauth status from runtime.
    #[error("unable to extract bioauth status: {0}")]
    UnableToGetStatus(sp_api::ApiError),
}

impl<Block: BlockT, Client, Id, Timestamp> AuthorizationVerifier<Block, Client, Id, Timestamp> {
    /// Create a new [`AuthorizationVerifier`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_types: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, Id, Timestamp> Clone
    for AuthorizationVerifier<Block, Client, Id, Timestamp>
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_types: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, Id, Timestamp> crate::AuthorizationVerifier
    for AuthorizationVerifier<Block, Client, Id, Timestamp>
where
    Client: ProvideRuntimeApi<Block>,
    Client::Api: BioauthFlowApi<Block, Id, Timestamp>,
    Id: codec::Encode + PartialEq,
    Timestamp: codec::Decode + PartialEq,
{
    type Error = AuthorizationVerifierError;
    type Block = Block;
    type PublicKeyType = Id;

    fn is_authorized(
        &self,
        at: &BlockId<Self::Block>,
        id: &Self::PublicKeyType,
    ) -> Result<bool, Self::Error> {
        let status = self
            .client
            .runtime_api()
            .bioauth_status(at, id)
            .map_err(AuthorizationVerifierError::UnableToGetStatus)?;

        let is_authorized = matches!(status, bioauth_flow_api::BioauthStatus::Active { .. });

        Ok(is_authorized)
    }
}
