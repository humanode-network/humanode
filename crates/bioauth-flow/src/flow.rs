//! Handle rpc endpoints

use std::ops::Deref;
use std::sync::Arc;

use jsonrpc_core::{Error as RpcError, ErrorCode};
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use robonode_client::{
    AuthenticateError, AuthenticateRequest, EnrollError, EnrollRequest, Error as RobonodeError,
    GetFacetecDeviceSdkParamsError, GetFacetecSessionTokenError,
};
use tracing::*;

use crate::{
    rpc::FacetecDeviceSdkParams,
    transaction_manager::{TransactionError, TransactionManager},
};

/// Errors that may occur during the bioauth flow.
#[derive(thiserror::Error, Debug)]
pub enum BioauthError {
    /// A transaction error.
    #[error("Transaction failed: {0}")]
    Transaction(#[from] TransactionError),
    /// Failed to get FaceTec sdk params.
    #[error("Failed to get FaceTec sdk params: {0}")]
    FacetecDeviceSdkParams(#[from] RobonodeError<GetFacetecDeviceSdkParamsError>),
    /// Failed to get FaceTec session token.
    #[error("Failed to get FaceTec session token: {0}")]
    GetFacetecSessionToken(#[from] RobonodeError<GetFacetecSessionTokenError>),
    /// Failed to authenticate.
    #[error("Failed to authenticate: {0}")]
    Authenticate(#[from] RobonodeError<AuthenticateError>),
    /// Failed to enroll.
    #[error("Failed to enroll: {0}")]
    Enroll(#[from] RobonodeError<EnrollError>),
    /// Signer error.
    #[error("Signer error has occurred: {0}")]
    Signer(String),
}

impl From<BioauthError> for RpcError {
    fn from(val: BioauthError) -> RpcError {
        RpcError {
            code: ErrorCode::ServerError(1),
            message: val.to_string(),
            data: None,
        }
    }
}

/// Signer provides signatures for the data.
#[async_trait::async_trait]
pub trait Signer<S> {
    /// Signature error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Sign the provided data and return the signature, or an error if the signing fails.
    async fn sign<'a, D>(&self, data: D) -> Result<S, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// Interface for handling bioauth rpc.
#[async_trait::async_trait]
pub trait BioauthFlow {
    /// Get the FaceTec Device SDK parameters to use at the device.
    async fn get_facetec_device_sdk_params(&self) -> Result<FacetecDeviceSdkParams, BioauthError>;

    /// Get the FaceTec Session Token.
    async fn get_facetec_session_token(&self) -> Result<String, BioauthError>;

    /// Authenticate given liveness data.
    async fn authenticate(&self, liveness_data: LivenessData) -> Result<(), BioauthError>;

    /// Enroll with given liveness data.
    async fn enroll(&self, liveness_data: LivenessData) -> Result<(), BioauthError>;
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
    async fn sign(
        &self,
        liveness_data: &LivenessData,
    ) -> Result<(OpaqueLivenessData, Vec<u8>), BioauthError> {
        let opaque_liveness_data = OpaqueLivenessData::from(liveness_data);

        let signature = self
            .validator_signer
            .sign(&opaque_liveness_data)
            .await
            .map_err(|e| BioauthError::Signer(e.to_string()))?;

        Ok((opaque_liveness_data, signature))
    }
}

#[async_trait::async_trait]
impl<RC, VPK, VS, M> BioauthFlow for Flow<RC, VPK, VS, M>
where
    RC: Deref<Target = robonode_client::Client> + Send + Sync,
    VS: Signer<Vec<u8>> + Send + Sync,
    <VS as Signer<Vec<u8>>>::Error: std::error::Error + 'static,
    VPK: AsRef<[u8]> + Send + Sync,
    M: TransactionManager + Send + Sync,
{
    async fn get_facetec_device_sdk_params(&self) -> Result<FacetecDeviceSdkParams, BioauthError> {
        let res = self.robonode_client.get_facetec_device_sdk_params().await?;
        Ok(res)
    }

    async fn get_facetec_session_token(&self) -> Result<String, BioauthError> {
        let res = self.robonode_client.get_facetec_session_token().await?;
        Ok(res.session_token)
    }

    async fn authenticate(&self, liveness_data: LivenessData) -> Result<(), BioauthError> {
        info!("Bioauth flow - authentication in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await?;

        let response = self
            .robonode_client
            .authenticate(AuthenticateRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await?;

        info!("Bioauth flow - authentication complete");

        info!(message = "We've obtained an auth ticket", auth_ticket = ?response.auth_ticket);

        self.transaction_manager
            .submit_authenticate(response)
            .await?;

        info!("Bioauth flow - authenticate transaction complete");

        Ok(())
    }

    async fn enroll(&self, liveness_data: LivenessData) -> Result<(), BioauthError> {
        info!("Bioauth flow - enrolling in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await?;

        self.robonode_client
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
                public_key: self.validator_public_key.as_ref().as_ref(),
            })
            .await?;

        info!("Bioauth flow - enrolling complete");

        Ok(())
    }
}

/// Handle the bioauth error in a user-friendly way.
#[allow(dead_code)]
fn handle_error(error: &anyhow::Error) -> (String, bool) {
    let face_scan_rejected_message = "the face scan was rejected, this is likely caused by a failed liveness check, so please try again; changing lighting conditions or using a different phone can help";

    if let Some(error) = error.downcast_ref::<RobonodeError<EnrollError>>() {
        match error {
            RobonodeError::Call(EnrollError::PersonAlreadyEnrolled) => {
                ("you have already enrolled".to_owned(), false)
            }
            RobonodeError::Call(EnrollError::PublicKeyAlreadyUsed) => (
                "the validator key you supplied was already used".to_owned(),
                false,
            ),
            RobonodeError::Call(EnrollError::FaceScanRejected) => {
                (face_scan_rejected_message.to_owned(), true)
            }
            RobonodeError::Call(EnrollError::InvalidLivenessData) => {
                ("the provided liveness data was invalid".to_owned(), true)
            }
            RobonodeError::Call(EnrollError::InvalidPublicKey) => {
                ("the public key was invalid".to_owned(), false)
            }
            RobonodeError::Call(EnrollError::LogicInternal) => {
                ("an internal logic error has occured".to_owned(), true)
            }
            RobonodeError::Call(EnrollError::UnknownCode(error_code)) => (
                format!(
                    "an unknown error code received from the server: {}",
                    error_code
                ),
                false,
            ),
            RobonodeError::Call(EnrollError::Unknown(err)) => (err.clone(), true),
            RobonodeError::Reqwest(err) => (err.to_string(), err.is_timeout()),
        }
    } else if let Some(error) = error.downcast_ref::<RobonodeError<AuthenticateError>>() {
        match error {
            RobonodeError::Reqwest(err) => (err.to_string(), err.is_timeout()),
            RobonodeError::Call(AuthenticateError::InvalidLivenessData) => {
                ("the provided liveness data was invalid".to_owned(), true)
            }
            RobonodeError::Call(AuthenticateError::PersonNotFound) => (
                "we were unable to find you in the system; have you already enrolled?".to_owned(),
                true,
            ),
            RobonodeError::Call(AuthenticateError::FaceScanRejected) => {
                (face_scan_rejected_message.to_owned(), true)
            }
            RobonodeError::Call(AuthenticateError::LogicInternal) => {
                ("an internal logic error has occured".to_owned(), true)
            }
            RobonodeError::Call(AuthenticateError::UnknownCode(error_code)) => (
                format!(
                    "an unknown error code received from the server: {}",
                    error_code
                ),
                false,
            ),
            RobonodeError::Call(AuthenticateError::Unknown(err)) => (err.clone(), true),
            RobonodeError::Call(AuthenticateError::SignatureInvalid) => {
("the validator key used for authentication does not match the one used during enroll; you have likely used a different mnemonic, but you have to use the same one, otherwise you will be unable to authenticate; you have saved the mnemonic somewhere as requested, right? ;) if you've lost your menmonic you will be unable to continue.".to_owned(), true)
}
        }
    } else {
        (error.to_string(), false)
    }
}
