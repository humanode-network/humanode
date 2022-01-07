//! Handle rpc endpoints

use std::ops::Deref;
use std::sync::Arc;

use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode;
use primitives_liveness_data::LivenessData;
use primitives_liveness_data::OpaqueLivenessData;
use robonode_client::AuthenticateRequest;
use robonode_client::EnrollRequest;

use crate::{rpc::FacetecDeviceSdkParams, transaction_manager::TransactionManager};

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

/// Interface for handling bioauth rpc.
#[async_trait::async_trait]
pub trait BioauthFlow {
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
pub struct Flow<RC, VPK, VS, M> {
    /// The transaction manager to use.
    pub transaction_manager: M,
    /// The robonode client.
    pub robonode_client: RC,
    /// The type used to encode the public key.
    pub validator_public_key: Arc<VPK>,
    /// The type that provides signing with the validator private key.
    pub validator_signer: Arc<VS>,
}

impl<RC, VPK, VS, M> Flow<RC, VPK, VS, M>
where
    VS: Signer<Vec<u8>>,
    <VS as Signer<Vec<u8>>>::Error: std::error::Error + 'static,
{
    /// Return the opaque liveness data and corresponding signature.
    async fn sign(&self, liveness_data: &LivenessData) -> (OpaqueLivenessData, Vec<u8>) {
        let opaque_liveness_data = OpaqueLivenessData::from(liveness_data);

        let signature = self
            .validator_signer
            .sign(&opaque_liveness_data)
            .await
            .unwrap();

        (opaque_liveness_data, signature)
    }
}

#[async_trait::async_trait]
impl<RC, VPK, VS, M> BioauthFlow for Flow<RC, VPK, VS, M>
where
    Self: Send + Sync,
    RC: Deref<Target = robonode_client::Client> + Send + Sync,
    VS: Signer<Vec<u8>> + Send + Sync,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    VPK: AsRef<[u8]> + Send + Sync,
    M: TransactionManager + Send + Sync,
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
        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await;

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
        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await;

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
