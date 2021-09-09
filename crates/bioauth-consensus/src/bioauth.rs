//! Bioauth pallet integration.

use pallet_bioauth::BioauthApi;
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::{marker::PhantomData, sync::Arc};

/// Provides an authorization verifier on top of stored auth tickets.
#[derive(Debug)]
pub struct AuthorizationVerifier<Block: BlockT, Client, ValidatorPublicKey> {
    /// The client provides access to the runtime.
    client: Arc<Client>,
    /// The type of the block used in the chain.
    _phantom_block: PhantomData<Block>,
    /// The type of the validator public key used in the chain.
    _phantom_validator_public_key: PhantomData<ValidatorPublicKey>,
}

/// An error that can occur during aura authorization verification.
#[derive(Debug, thiserror::Error)]
pub enum AuraAuthorizationVerifierError {
    /// Something went wrong while extracting the stored auth tickets from the chain state via
    /// the runtime.
    #[error("unable to extract stored auth tickets: {0}")]
    UnableToExtractStoredAuthTickets(sp_api::ApiError),
}

impl<Block: BlockT, Client, ValidatorPublicKey>
    AuthorizationVerifier<Block, Client, ValidatorPublicKey>
{
    /// Create a new [`AuraAuthorizationVerifier`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _phantom_block: PhantomData,
            _phantom_validator_public_key: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, ValidatorPublicKey> Clone
    for AuthorizationVerifier<Block, Client, ValidatorPublicKey>
{
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            _phantom_block: PhantomData,
            _phantom_validator_public_key: PhantomData,
        }
    }
}

impl<Block: BlockT, Client, ValidatorPublicKey> crate::AuthorizationVerifier
    for AuthorizationVerifier<Block, Client, ValidatorPublicKey>
where
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    Client::Api: BioauthApi<Block, ValidatorPublicKey>,
    ValidatorPublicKey: codec::Decode + PartialEq,
{
    type Error = AuraAuthorizationVerifierError;
    type Block = Block;
    type PublicKeyType = ValidatorPublicKey;

    fn is_authorized(
        &self,
        at: &BlockId<Self::Block>,
        author_public_key: &Self::PublicKeyType,
    ) -> Result<bool, Self::Error> {
        // Get current stored tickets.
        let stored_tickets = self
            .client
            .runtime_api()
            .stored_auth_tickets(at)
            .map_err(AuraAuthorizationVerifierError::UnableToExtractStoredAuthTickets)?;

        let is_authorized = stored_tickets
            .iter()
            .any(|ticket| &ticket.public_key == author_public_key);

        Ok(is_authorized)
    }
}
