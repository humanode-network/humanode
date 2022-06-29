//! Interface for getting chain properties.

use std::{marker::PhantomData, sync::Arc};

use chain_properties_api::ChainPropertiesApi;
use sp_api::{BlockT, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;

/// Provides access to chain properties.
pub struct ChainProperties<Client, Block> {
    /// The substrate client, provides access to the runtime APIs.
    client: Arc<Client>,
    /// The phantom types.
    phantom_types: PhantomData<Block>,
}

impl<Client, Block> ChainProperties<Client, Block> {
    /// Create a new [`ChainProperties`].
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            phantom_types: PhantomData,
        }
    }
}

impl<Client, Block> ChainProperties<Client, Block>
where
    Client: Send + Sync + 'static,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client::Api: ChainPropertiesApi<Block>,
    Block: BlockT,
{
    /// Get chain Ss58 prefix.
    pub fn ss58_prefix(&self) -> u16 {
        // Extract an id of the genesis block.
        let at = sp_api::BlockId::Hash(self.client.info().genesis_hash);

        self.client.runtime_api().ss58_prefix(&at).unwrap()
    }
}
