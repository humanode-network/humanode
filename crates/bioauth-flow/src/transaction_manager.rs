//! Handles any transactions that need to occur during the bioauth flow.
use std::sync::Arc;

use sc_client_api::UsageProvider;
use sc_transaction_pool_api::TransactionPool;

use robonode_client::AuthenticateResponse;

/// Errors that may occur from a transaction manager.
#[derive(thiserror::Error, Debug)]
pub enum TransactionError {
    /// Authentication failed
    // TODO: Do we want to include the original error here?
    #[error("Authenticate transaction failed.")]
    AuthenticateFailed,
}

/// Interface for rpc transactions.
#[async_trait::async_trait]
pub trait TransactionManager {
    /// Submit an authenticate transaction.
    async fn submit_authenticate(
        &self,
        response: AuthenticateResponse,
    ) -> Result<(), TransactionError>;
}

/// Implementation for rpc transactions.
pub struct Manager<C, TP> {
    /// The client to use for transactions.
    pub client: Arc<C>,
    /// The transaction pool to use.
    pub pool: Arc<TP>,
}

#[async_trait::async_trait]
impl<C, TP> TransactionManager for Manager<C, TP>
where
    TP: TransactionPool + Send + Sync,
    C: UsageProvider<<TP as TransactionPool>::Block> + Send + Sync,
    <<TP as TransactionPool>::Block as sp_runtime::traits::Block>::Extrinsic:
        From<humanode_runtime::UncheckedExtrinsic>,
{
    async fn submit_authenticate(
        &self,
        response: AuthenticateResponse,
    ) -> Result<(), TransactionError> {
        let authenticate = pallet_bioauth::Authenticate {
            ticket: response.auth_ticket.into(),
            ticket_signature: response.auth_ticket_signature.into(),
        };

        let call = pallet_bioauth::Call::authenticate { req: authenticate };

        let ext = humanode_runtime::UncheckedExtrinsic::new_unsigned(call.into());

        let at = self.client.usage_info().chain.best_hash;

        self.pool
            .submit_and_watch(
                &sp_runtime::generic::BlockId::Hash(at),
                sp_runtime::transaction_validity::TransactionSource::Local,
                ext.into(),
            )
            .await
            .map_err(|_| TransactionError::AuthenticateFailed)?;

        Ok(())
    }
}
