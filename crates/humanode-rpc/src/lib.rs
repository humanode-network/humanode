//! RPC subsystem instantiation logic.

use std::sync::Arc;

use bioauth_flow::{Bioauth, BioauthApi, Signer, ValidatorKeyExtractorT};
use humanode_runtime::{opaque::Block, AccountId, Balance, Index, UnixMilliseconds};
use sc_client_api::UsageProvider;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::{Encode, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// RPC subsystem dependencies.
pub struct Deps<C, P, VS, VKE> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The type that provides signing with the validator private key.
    pub validator_signer: Option<Arc<VS>>,
    /// Whether to deny unsafe calls.
    pub deny_unsafe: DenyUnsafe,
    /// An ready robonode API client to tunnel the calls to.
    pub robonode_client: Arc<robonode_client::Client>,
    /// Extracts the currently used validator key.
    pub validator_key_extractor: VKE,
}

/// Instantiate all RPC extensions.
pub fn create<C, P, VS, VKE>(
    deps: Deps<C, P, VS, VKE>,
) -> jsonrpc_core::IoHandler<sc_rpc_api::Metadata>
where
    VS: Signer<Vec<u8>> + Send + Sync + 'static,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    VKE::PublicKeyType: Send + Sync,
    C: UsageProvider<Block>,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: bioauth_flow_api::BioauthFlowApi<Block, VKE::PublicKeyType, UnixMilliseconds>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool<Block = Block> + 'static,
    VKE: ValidatorKeyExtractorT + Send + Sync + 'static,
    VKE::PublicKeyType: Encode,
    VKE::PublicKeyType: AsRef<[u8]>,
    VKE::Error: std::fmt::Debug,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let Deps {
        client,
        pool,
        validator_signer,
        deny_unsafe,
        robonode_client,
        validator_key_extractor,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        Arc::clone(&client),
        Arc::clone(&pool),
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        Arc::clone(&client),
    )));

    if let Some(validator_signer) = validator_signer {
        io.extend_with(BioauthApi::to_delegate(Bioauth::new(
            robonode_client,
            validator_signer,
            client,
            pool,
            validator_key_extractor,
        )));
    }

    io
}
