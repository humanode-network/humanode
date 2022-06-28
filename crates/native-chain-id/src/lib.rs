use std::{marker::PhantomData, sync::Arc};

use native_chain_id_api::NativeChainIdApi;
use sp_api::{BlockT, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;

pub struct NativeChainId<Client, Block> {
    client: Arc<Client>,
    phantom_types: PhantomData<Block>,
}

impl<Client, Block> NativeChainId<Client, Block> {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            phantom_types: PhantomData,
        }
    }
}

impl<Client, Block> NativeChainId<Client, Block>
where
    Client: Send + Sync + 'static,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client::Api: NativeChainIdApi<Block>,
    Block: BlockT,
{
    pub fn get(&self) -> u16 {
        // Extract an id of the genesis block.
        let at = sp_api::BlockId::Hash(self.client.info().genesis_hash);

        self.client.runtime_api().get(&at).unwrap()
    }
}
