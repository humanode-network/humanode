//! Runtime integration.

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

/// An error that can occur during aura authorization verification.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationVerifierError {
    /// Something went wrong while extracting the authorized ids from the chain state via
    /// the runtime.
    #[error("unable to extract authorized ids from the chain state: {0}")]
    UnableToExtractAuthorizedIds(sp_api::ApiError),
}

impl<Block: BlockT, Client, Id> AuthorizationVerifier<Block, Client, Id> {
    /// Create a new [`AuraAuthorizationVerifier`].
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
    Id: codec::Decode + PartialEq,
{
    type Error = AuthorizationVerifierError;
    type Block = Block;
    type PublicKeyType = Id;

    fn is_authorized(
        &self,
        at: &BlockId<Self::Block>,
        id: &Self::PublicKeyType,
    ) -> Result<bool, Self::Error> {
        let authorized_ids = self
            .client
            .runtime_api()
            .ids(at)
            .map_err(AuthorizationVerifierError::UnableToExtractAuthorizedIds)?;

        let is_authorized = authorized_ids
            .iter()
            .any(|authorized_id| authorized_id == id);

        Ok(is_authorized)
    }
}
