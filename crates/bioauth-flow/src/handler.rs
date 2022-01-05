//! Handle rpc endpoints

use std::ops::Deref;
use std::sync::Arc;

use futures::channel::oneshot;
use futures::lock::BiLock;
use futures::FutureExt;
use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode;
use jsonrpc_derive::rpc;
use primitives_liveness_data::LivenessData;
use primitives_liveness_data::OpaqueLivenessData;
use robonode_client::AuthenticateRequest;
use robonode_client::EnrollRequest;
use sc_transaction_pool_api::TransactionPool;
use serde_json::{Map, Value};

use humanode_runtime::{self, opaque::Block, RuntimeApi};

use crate::flow::LivenessDataProvider;
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

#[async_trait::async_trait]
pub trait RpcHandler {
    /// Get the FaceTec Device SDK parameters to use at the device.
    async fn get_facetec_device_sdk_params(&self) -> Result<FacetecDeviceSdkParams, RpcError>;

    /// Get the FaceTec Session Token.
    async fn get_facetec_session_token(&self) -> Result<String, RpcError>;

    async fn authenticate(&self, liveness_data: LivenessData) -> Result<(), RpcError>;

    async fn enroll(&self, liveness_data: LivenessData) -> Result<(), RpcError>;
}

/// The underlying implementation of the RPC part, extracted into a subobject to work around
/// the common pitfall with the poor async engines implementations of requiring future objects to
/// be static.
/// Stop it people, why do you even use Rust if you do things like this? Ffs...
/// See https://github.com/paritytech/jsonrpc/issues/580
pub struct Handler<RC, VPK, VS, TP> {
    /// The robonode client.
    pub client: RC,
    /// The type used to encode the public key.
    pub validator_public_key: Arc<VPK>,
    /// The type that provides signing with the validator private key.
    pub validator_signer: Arc<VS>,
    pub transaction_pool: Arc<TP>,
}

impl<RC, VPK, VS, TP> Handler<RC, VPK, VS, TP>
where
    RC: Deref<Target = robonode_client::Client>,
{
    /// Get the FaceTec Device SDK parameters to use at the device.
    async fn get_facetec_device_sdk_params(&self) -> Result<FacetecDeviceSdkParams, RpcError> {
        let res = self
            .client
            .get_facetec_device_sdk_params()
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;
        Ok(res)
    }

    /// Get the FaceTec Session Token.
    async fn get_facetec_session_token(&self) -> Result<String, RpcError> {
        let res = self
            .client
            .get_facetec_session_token()
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;
        Ok(res.session_token)
    }
}

impl<RC, VPK, VS, TP> Handler<RC, VPK, VS, TP>
where
    RC: Deref<Target = robonode_client::Client>,
    VS: Signer<Vec<u8>>,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    VPK: AsRef<[u8]>,
    TP: TransactionPool,
{
    /// Authenticate
    async fn authenticate(&self, liveness_data: LivenessData) -> Result<(), RpcError> {
        let opaque_liveness_data = OpaqueLivenessData::from(&liveness_data);

        let signature = self
            .validator_signer
            .sign(&opaque_liveness_data)
            .await
            .unwrap();

        let response = self
            .client
            .authenticate(AuthenticateRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await
            .unwrap();

        // let authenticate = pallet_bioauth::Authenticate {
        // ticket: response.auth_ticket.into(),
        // ticket_signature: response.auth_ticket_signature.into(),
        // };
        // let call = pallet_bioauth::Call::authenticate { req: authenticate };

        // let ext = humanode_runtime::UncheckedExtrinsic::new_unsigned(call.into());

        // let at = self.client.chain_info().best_hash;
        // self.transaction_pool
        // .submit_and_watch(
        // &sp_runtime::generic::BlockId::Hash(at),
        // sp_runtime::transaction_validity::TransactionSource::Local,
        // ext.into(),
        // )
        // .await
        // .unwrap();

        Ok(())
    }
}

impl<RC, VPK, VS, TP> Handler<RC, VPK, VS, TP>
where
    RC: Deref<Target = robonode_client::Client>,
    VS: Signer<Vec<u8>>,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    VPK: AsRef<[u8]>,
{
    /// Enroll
    async fn enroll(&self, liveness_data: LivenessData) -> Result<(), RpcError> {
        let opaque_liveness_data = OpaqueLivenessData::from(&liveness_data);

        let signature = self
            .validator_signer
            .sign(&opaque_liveness_data)
            .await
            .unwrap();

        self.client
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

#[async_trait::async_trait]
impl<RC, VPK, VS, TP> RpcHandler for Handler<RC, VPK, VS, TP>
where
    Self: Send + Sync,
    RC: Deref<Target = robonode_client::Client>,
    VS: Signer<Vec<u8>>,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    VPK: AsRef<[u8]> + Send,
    TP: TransactionPool,
{
    /// Get the FaceTec Device SDK parameters to use at the device.
    async fn get_facetec_device_sdk_params(&self) -> Result<FacetecDeviceSdkParams, RpcError> {
        // self.get_facetec_device_sdk_params().await
        todo!();
    }

    /// Get the FaceTec Session Token.
    async fn get_facetec_session_token(&self) -> Result<String, RpcError> {
        // self.get_facetec_session_token().await
        todo!();
    }

    async fn authenticate(&self, liveness_data: LivenessData) -> Result<(), RpcError> {
        // self.authenticate(liveness_data).await
        todo!();
    }

    async fn enroll(&self, liveness_data: LivenessData) -> Result<(), RpcError> {
        // self.enroll(liveness_data).await
        todo!();
    }
}
