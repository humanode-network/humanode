//! RPC subsystem instantiation logic.

use std::sync::Arc;

use bioauth_flow::{
    rpc::{Bioauth, BioauthApi, LivenessDataTxSlot, ValidatorKeyExtractorT},
    Signer,
};
use humanode_runtime::{opaque::Block, AccountId, Balance, Index, UnixMilliseconds};
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::{BlockT, Encode, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// The humanode runtime specific transaction manager.
struct TransactionManager<C, P> {
    /// The substrate client, provides access to the runtime APIs.
    client: Arc<C>,
    /// The transaction pool, used to submit transactions.
    pool: Arc<P>,
}

#[async_trait::async_trait]
impl<C, P> bioauth_flow::TransactionManager for TransactionManager<C, P>
where
    C: HeaderBackend<P::Block> + Send + Sync,
    P: TransactionPool + Send + Sync,
    <<P as TransactionPool>::Block as BlockT>::Extrinsic:
        From<humanode_runtime::UncheckedExtrinsic>,
{
    type Error = P::Error;

    async fn submit_authenticate<OpaqueAuthTicket, Commitment>(
        &self,
        auth_ticket: OpaqueAuthTicket,
        ticket_signature: Commitment,
    ) -> Result<(), Self::Error>
    where
        OpaqueAuthTicket: AsRef<[u8]> + Send + Sync,
        Commitment: AsRef<[u8]> + Send + Sync,
    {
        let authenticate = pallet_bioauth::Authenticate {
            ticket: auth_ticket.as_ref().to_vec().into(),
            ticket_signature: ticket_signature.as_ref().to_vec(),
        };

        let call = pallet_bioauth::Call::authenticate { req: authenticate };

        let ext = Block::UncheckedExtrinsic::new_unsigned(call.into());

        let at = self.client.info().best_hash;

        self.pool
            .submit_and_watch(
                &sp_runtime::generic::BlockId::Hash(at),
                sp_runtime::transaction_validity::TransactionSource::Local,
                ext.into(),
            )
            .await?;

        Ok(())
    }
}

/// RPC subsystem dependencies.
pub struct Deps<C, P, VKE, VS> {
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
    /// The type that provides signing with the validator private key.
    pub validator_signer: Option<Arc<VS>>,
}

/// Instantiate all RPC extensions.
pub fn create<C, P, VKE, VS>(
    deps: Deps<C, P, VKE, VS>,
) -> jsonrpc_core::IoHandler<sc_rpc_api::Metadata>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: bioauth_flow_api::BioauthFlowApi<Block, VKE::PublicKeyType, UnixMilliseconds>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool<Block = Block> + 'static,
    VKE: ValidatorKeyExtractorT + Send + Sync + 'static,
    VKE::PublicKeyType: Encode + AsRef<[u8]> + Send + Sync,
    VKE::Error: std::fmt::Debug,
    VS: Signer<Vec<u8>> + Send + Sync + 'static,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
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
        validator_signer,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        Arc::clone(&client),
        Arc::clone(&pool),
        deny_unsafe,
    )));

    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        Arc::clone(&client),
    )));

    io.extend_with(BioauthApi::to_delegate(Bioauth::new(
        robonode_client,
        bioauth_flow_slot,
        validator_key_extractor,
        validator_signer,
        Arc::clone(&client),
        TransactionManager {
            client: Arc::clone(&client),
            pool: Arc::clone(&pool),
        },
    )));

    io
}
