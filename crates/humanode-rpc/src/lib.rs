//! RPC subsystem instantiation logic.

use std::sync::Arc;

use bioauth_flow::{
    handler::{Handler, Signer},
    rpc::{Bioauth, BioauthApi},
};
use humanode_runtime::{opaque::Block, AccountId, Balance, Index};
use sc_client_api::UsageProvider;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// RPC subsystem dependencies.
pub struct Deps<C, P, VPK, VS> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The type used to encode the public key.
    pub validator_public_key: Option<Arc<VPK>>,
    /// The type that provides signing with the validator private key.
    pub validator_signer: Option<Arc<VS>>,
    /// Whether to deny unsafe calls.
    pub deny_unsafe: DenyUnsafe,
    /// An ready robonode API client to tunnel the calls to.
    pub robonode_client: Arc<robonode_client::Client>,
}

/// Instantiate all RPC extensions.
pub fn create<C, P, VPK, VS>(
    deps: Deps<C, P, VPK, VS>,
) -> jsonrpc_core::IoHandler<sc_rpc_api::Metadata>
where
    VS: Signer<Vec<u8>> + Send + Sync + 'static,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    VPK: AsRef<[u8]> + Send + Sync + 'static,
    C: UsageProvider<Block>,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool<Block = Block> + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let Deps {
        client,
        pool,
        validator_public_key,
        validator_signer,
        deny_unsafe,
        robonode_client,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        Arc::clone(&client),
        Arc::clone(&pool),
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        Arc::clone(&client),
    )));

    if let (Some(validator_public_key), Some(validator_signer)) =
        (validator_public_key, validator_signer)
    {
        io.extend_with(BioauthApi::to_delegate(Bioauth::new(Handler {
            robonode_client,
            validator_public_key,
            validator_signer,
            transaction_manager: bioauth_flow::handler::TransactionManager { client, pool },
        })));
    }

    io
}
