//! RPC subsystem instantiation logic.

use std::sync::Arc;

use bioauth_flow::rpc::{Bioauth, BioauthApi, LivenessDataTxSlot, ValidatorKeyExtractorT};
use humanode_runtime::{opaque::Block, AccountId, Balance, Index, UnixMilliseconds};
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::{Encode, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// RPC subsystem dependencies.
pub struct Deps<C, P, VKE> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls.
    pub deny_unsafe: DenyUnsafe,
    /// An ready robonode API client to tunnel the calls to.
    pub robonode_client: Arc<robonode_client::Client>,
    /// The liveness data tx slot to use in the bioauth flow RPC.
    pub bioauth_flow_slot: Arc<LivenessDataTxSlot>,
    /// Extracts the currently used validator key.
    pub validator_key_extractor: VKE,
}

/// Instantiate all RPC extensions.
pub fn create<C, P, VKE>(deps: Deps<C, P, VKE>) -> jsonrpc_core::IoHandler<sc_rpc_api::Metadata>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: bioauth_flow_api::BioauthFlowApi<Block, VKE::PublicKeyType, UnixMilliseconds>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
    VKE: ValidatorKeyExtractorT + Send + Sync + 'static,
    VKE::PublicKeyType: Encode,
    VKE::Error: std::fmt::Debug,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let Deps {
        client,
        pool,
        deny_unsafe,
        robonode_client,
        bioauth_flow_slot,
        validator_key_extractor,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        Arc::clone(&client),
        pool,
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        Arc::clone(&client),
    )));

    io.extend_with(BioauthApi::to_delegate(Bioauth::new(
        robonode_client,
        bioauth_flow_slot,
        validator_key_extractor,
        Arc::clone(&client),
    )));

    io
}
