//! Handle rpc endpoints

use std::ops::Deref;
use std::sync::Arc;

use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode;
use primitives_liveness_data::LivenessData;
use primitives_liveness_data::OpaqueLivenessData;
use robonode_client::AuthenticateRequest;
use robonode_client::AuthenticateResponse;
use robonode_client::EnrollRequest;
use sc_client_api::UsageProvider;
use sc_transaction_pool_api::TransactionPool;

use crate::rpc::FacetecDeviceSdkParams;

/// Signer provides signatures for the data.
#[async_trait::async_trait]
pub trait Signer<S> {
    /// Signature error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Sign the provided data and return the signature, or an error if the siging fails.
    async fn sign<'a, D>(&self, data: D) -> Result<S, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// Interface for rpc transactions.
#[async_trait::async_trait]
pub trait RpcTransactionManager {
    /// Submit an authenticate transaction.
    async fn submit_authenticate(&self, response: AuthenticateResponse);
}

/// Implementation for rpc transactions.
pub struct TransactionManager<C, TP> {
    /// The client to use for transactions.
    pub client: Arc<C>,
    /// The transaction pool to use.
    pub pool: Arc<TP>,
}

#[async_trait::async_trait]
impl<C, TP> RpcTransactionManager for TransactionManager<C, TP>
where
    TP: TransactionPool + Send + Sync,
    C: UsageProvider<<TP as TransactionPool>::Block> + Send + Sync,
    <<TP as TransactionPool>::Block as sp_runtime::traits::Block>::Extrinsic:
        From<humanode_runtime::UncheckedExtrinsic>,
{
    async fn submit_authenticate(&self, response: AuthenticateResponse) {
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
            .unwrap();
    }
}

/// Interface for handling bioauth rpc.
#[async_trait::async_trait]
pub trait RpcHandler {
    /// Get the FaceTec Device SDK parameters to use at the device.
    async fn get_facetec_device_sdk_params(&self) -> Result<FacetecDeviceSdkParams, RpcError>;

    /// Get the FaceTec Session Token.
    async fn get_facetec_session_token(&self) -> Result<String, RpcError>;

    /// Authenticate given liveness data.
    async fn authenticate(&self, liveness_data: LivenessData) -> Result<(), RpcError>;

    /// Enroll with given liveness data.
    async fn enroll(&self, liveness_data: LivenessData) -> Result<(), RpcError>;
}

/// Implementation for handling bioauth rpc.
pub struct Handler<RC, VPK, VS, M> {
    /// The transaction manager to use.
    pub transaction_manager: M,
    /// The robonode client.
    pub robonode_client: RC,
    /// The type used to encode the public key.
    pub validator_public_key: Arc<VPK>,
    /// The type that provides signing with the validator private key.
    pub validator_signer: Arc<VS>,
}

#[async_trait::async_trait]
impl<RC, VPK, VS, M> RpcHandler for Handler<RC, VPK, VS, M>
where
    Self: Send + Sync,
    RC: Deref<Target = robonode_client::Client> + Send + Sync,
    VS: Signer<Vec<u8>> + Send + Sync,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    VPK: AsRef<[u8]> + Send + Sync,
    M: RpcTransactionManager + Send + Sync,
{
    async fn get_facetec_device_sdk_params(&self) -> Result<FacetecDeviceSdkParams, RpcError> {
        let res = self
            .robonode_client
            .get_facetec_device_sdk_params()
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;
        Ok(res)
    }

    async fn get_facetec_session_token(&self) -> Result<String, RpcError> {
        let res = self
            .robonode_client
            .get_facetec_session_token()
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;
        Ok(res.session_token)
    }

    async fn authenticate(&self, liveness_data: LivenessData) -> Result<(), RpcError> {
        let opaque_liveness_data = OpaqueLivenessData::from(&liveness_data);

        let signature = self
            .validator_signer
            .sign(&opaque_liveness_data)
            .await
            .unwrap();

        let response = self
            .robonode_client
            .authenticate(AuthenticateRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await
            .unwrap();

        self.transaction_manager.submit_authenticate(response).await;

        Ok(())
    }

    async fn enroll(&self, liveness_data: LivenessData) -> Result<(), RpcError> {
        let opaque_liveness_data = OpaqueLivenessData::from(&liveness_data);

        let signature = self
            .validator_signer
            .sign(&opaque_liveness_data)
            .await
            .unwrap();

        self.robonode_client
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
                public_key: self.validator_public_key.as_ref().as_ref(),
            })
            .await
            .unwrap();
        Ok(())
    }
}
