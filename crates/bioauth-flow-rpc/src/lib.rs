//! The bioauth flow RPC implementation.
//!
//! It is the logic of communication between the humanode (aka humanode-peer),
//! the app on the handheld device that performs the biometric capture,
//! and the robonode server that issues auth tickets.

use std::sync::Arc;

use bioauth_flow_api::BioauthFlowApi;
use bioauth_keys::traits::KeyExtractor as KeyExtractorT;
use errors::{
    authenticate::Error as AuthenticateError, authenticate_v2::Error as AuthenticateV2Error,
    enroll::Error as EnrollError, enroll_v2::Error as EnrollV2Error,
    get_facetec_device_sdk_params::Error as GetFacetecDeviceSdkParamsError,
    get_facetec_session_token::Error as GetFacetecSessionToken, shared::FlowBaseError,
    sign::Error as SignError, status::Error as StatusError,
};
use futures::StreamExt;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use robonode_client::{AuthenticateRequest, AuthenticateResponse, EnrollRequest, EnrollResponse};
use rpc_deny_unsafe::DenyUnsafe;
use sc_transaction_pool_api::{TransactionPool as _, TransactionStatus, TxHash};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sp_api::ProvideRuntimeApi as _;
use sp_blockchain::HeaderBackend as _;
use tracing::*;

mod config;
pub mod error_data;
mod errors;
pub mod signer;

pub use self::config::Config;
pub use self::signer::{Factory as SignerFactory, Signer};

/// The parameters necessary to initialize the FaceTec Device SDK.
type FacetecDeviceSdkParams = Map<String, Value>;

/// The bioauth status as used in the RPC.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BioauthStatus<Timestamp> {
    /// When the status can't be determined, but there was no error.
    /// Can happen if the validator key is absent.
    Unknown,
    /// There is no active authentication for the currently used validator key.
    Inactive,
    /// There is an active authentication for the currently used validator key.
    Active {
        /// The timestamp when the authentication will expire.
        expires_at: Timestamp,
    },
}

impl<T> From<bioauth_flow_api::BioauthStatus<T>> for BioauthStatus<T> {
    fn from(status: bioauth_flow_api::BioauthStatus<T>) -> Self {
        match status {
            bioauth_flow_api::BioauthStatus::Inactive => Self::Inactive,
            bioauth_flow_api::BioauthStatus::Active { expires_at } => Self::Active { expires_at },
        }
    }
}

/// The API exposed via JSON-RPC.
#[rpc(server)]
pub trait Bioauth<Timestamp, TxHash> {
    /// Get the configuration required for the Device SDK.
    #[method(name = "bioauth_getFacetecDeviceSdkParams")]
    async fn get_facetec_device_sdk_params(&self) -> RpcResult<FacetecDeviceSdkParams>;

    /// Get a FaceTec Session Token.
    #[method(name = "bioauth_getFacetecSessionToken")]
    async fn get_facetec_session_token(&self) -> RpcResult<String>;

    /// Get the current bioauth status.
    #[method(name = "bioauth_status")]
    async fn status(&self) -> RpcResult<BioauthStatus<Timestamp>>;

    /// Enroll with provided liveness data.
    #[method(name = "bioauth_enroll")]
    async fn enroll(&self, liveness_data: LivenessData) -> RpcResult<()>;

    /// Enroll with provided liveness data V2.
    #[method(name = "bioauth_enrollV2")]
    async fn enroll_v2(&self, liveness_data: LivenessData) -> RpcResult<EnrollV2Result>;

    /// Authenticate with provided liveness data.
    #[method(name = "bioauth_authenticate")]
    async fn authenticate(&self, liveness_data: LivenessData) -> RpcResult<TxHash>;

    /// Request auth ticket based on provided liveness data.
    #[method(name = "bioauth_authenticateV2")]
    async fn authenticate_v2(&self, liveness_data: LivenessData)
        -> RpcResult<AuthenticateV2Result>;
}

/// `enroll_v2` flow result.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrollV2Result {
    /// Scan result blob.
    pub scan_result_blob: Option<String>,
}

/// `authenticate_v2` related flow result.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateV2Result {
    /// An opaque auth ticket generated for this authentication attempt.
    pub auth_ticket: Box<[u8]>,
    /// The robonode signature for this opaque auth ticket.
    pub auth_ticket_signature: Box<[u8]>,
    /// Scan result blob.
    pub scan_result_blob: Option<String>,
}

/// The RPC implementation.
pub struct Bioauth<Config: self::Config> {
    /// The robonode client, used for fetching the FaceTec Session Token.
    pub robonode_client: Config::RobonodeClient,
    /// Provider of the local validator key.
    pub validator_key_extractor: Config::ValidatorKeyExtractor,
    /// The type that provides signing with the validator private key.
    pub validator_signer_factory: Config::ValidatorSignerFactory,
    /// The substrate client, provides access to the runtime APIs.
    pub client: Arc<Config::Client>,
    /// The transaction pool to use.
    pub pool: Arc<Config::TransactionPool>,
    /// Whether to deny unsafe calls or not.
    pub deny_unsafe: DenyUnsafe,
}

impl<Config: self::Config> Bioauth<Config> {
    /// Return the opaque liveness data and corresponding signature.
    async fn sign(
        &self,
        validator_key: <Config::ValidatorKeyExtractor as KeyExtractorT>::PublicKeyType,
        liveness_data: &LivenessData,
    ) -> Result<(OpaqueLivenessData, Vec<u8>), SignError> {
        let opaque_liveness_data = OpaqueLivenessData::from(liveness_data);
        let signer = self.validator_signer_factory.new_signer(validator_key);

        let signature = signer.sign(&opaque_liveness_data).await.map_err(|error| {
            tracing::error!(message = "Signing failed", ?error);
            SignError::SigningFailed
        })?;

        Ok((opaque_liveness_data, signature))
    }

    /// Do enroll with provided liveness data.
    async fn do_enroll(
        &self,
        liveness_data: LivenessData,
    ) -> Result<EnrollResponse, FlowBaseError<robonode_client::EnrollError>> {
        info!("Bioauth flow - enrolling in progress");

        let public_key =
            rpc_validator_key_logic::validator_public_key(&self.validator_key_extractor)
                .map_err(FlowBaseError::KeyExtraction)?;

        let (opaque_liveness_data, signature) = self
            .sign(public_key.clone(), &liveness_data)
            .await
            .map_err(FlowBaseError::Sign)?;

        let response = self
            .robonode_client
            .as_ref()
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
                public_key: public_key.as_ref(),
            })
            .await
            .map_err(FlowBaseError::RobonodeClient)?;

        info!("Bioauth flow - enrolling complete");

        Ok(response)
    }

    /// Do authenticate with provided liveness data.
    async fn do_authenticate(
        &self,
        liveness_data: LivenessData,
    ) -> Result<AuthenticateResponse, FlowBaseError<robonode_client::AuthenticateError>> {
        info!("Bioauth flow - authentication in progress");

        let public_key =
            rpc_validator_key_logic::validator_public_key(&self.validator_key_extractor)
                .map_err(FlowBaseError::KeyExtraction)?;

        let (opaque_liveness_data, signature) = self
            .sign(public_key, &liveness_data)
            .await
            .map_err(FlowBaseError::Sign)?;

        let response = self
            .robonode_client
            .as_ref()
            .authenticate(AuthenticateRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await
            .map_err(FlowBaseError::RobonodeClient)?;

        info!("Bioauth flow - authentication complete");

        Ok(response)
    }
}

#[jsonrpsee::core::async_trait]
impl<Config: self::Config> BioauthServer<Config::Timestamp, TxHash<Config::TransactionPool>>
    for Bioauth<Config>
{
    async fn get_facetec_device_sdk_params(&self) -> RpcResult<FacetecDeviceSdkParams> {
        let res = self
            .robonode_client
            .as_ref()
            .get_facetec_device_sdk_params()
            .await
            .map_err(GetFacetecDeviceSdkParamsError::Robonode)?;
        Ok(res)
    }

    async fn get_facetec_session_token(&self) -> RpcResult<String> {
        let res = self
            .robonode_client
            .as_ref()
            .get_facetec_session_token()
            .await
            .map_err(GetFacetecSessionToken::Robonode)?;
        Ok(res.session_token)
    }

    async fn status(&self) -> RpcResult<BioauthStatus<Config::Timestamp>> {
        let own_key =
            match rpc_validator_key_logic::validator_public_key(&self.validator_key_extractor) {
                Ok(v) => v,
                Err(rpc_validator_key_logic::Error::MissingValidatorKey) => {
                    return Ok(BioauthStatus::Unknown)
                }
                Err(rpc_validator_key_logic::Error::ValidatorKeyExtraction) => {
                    return Err(StatusError::ValidatorKeyExtraction.into())
                }
            };

        // Extract an id of the last imported block.
        let at = self.client.info().best_hash;

        let status = self
            .client
            .runtime_api()
            .bioauth_status(at, &own_key)
            .map_err(StatusError::RuntimeApi)?;

        Ok(status.into())
    }

    async fn enroll(&self, liveness_data: LivenessData) -> RpcResult<()> {
        self.deny_unsafe.check_if_safe()?;

        self.do_enroll(liveness_data).await.map_err(EnrollError)?;

        Ok(())
    }

    async fn enroll_v2(&self, liveness_data: LivenessData) -> RpcResult<EnrollV2Result> {
        self.deny_unsafe.check_if_safe()?;

        let EnrollResponse { scan_result_blob } =
            self.do_enroll(liveness_data).await.map_err(EnrollV2Error)?;

        Ok(EnrollV2Result { scan_result_blob })
    }

    async fn authenticate(
        &self,
        liveness_data: LivenessData,
    ) -> RpcResult<TxHash<Config::TransactionPool>> {
        self.deny_unsafe.check_if_safe()?;

        let errtype = |val: errors::authenticate::Error<
            <Config::TransactionPool as sc_transaction_pool_api::TransactionPool>::Error,
        >| val;

        let response = self
            .do_authenticate(liveness_data)
            .await
            .map_err(AuthenticateError::RobonodeRequest)
            .map_err(errtype)?;

        info!(message = "We've obtained an auth ticket", auth_ticket = ?response.auth_ticket);

        let at = self.client.info().best_hash;

        let ext = self
            .client
            .runtime_api()
            .create_authenticate_extrinsic(
                at,
                response.auth_ticket.into(),
                response.auth_ticket_signature.into(),
            )
            .map_err(AuthenticateError::RuntimeApi)
            .map_err(errtype)?;

        info!("Bioauth flow - submitting authenticate transaction");

        let tx_hash = self.pool.hash_of(&ext);

        let mut watch = self
            .pool
            .submit_and_watch(
                &sp_api::BlockId::Hash(at),
                sp_runtime::transaction_validity::TransactionSource::Local,
                ext,
            )
            .await
            .map_err(AuthenticateError::BioauthTx)
            .map_err(errtype)?
            .fuse();

        tokio::spawn(async move {
            loop {
                let maybe_tx_status = watch.next().await;

                match maybe_tx_status {
                    Some(TransactionStatus::Finalized((block_hash, _))) => {
                        info!(
                            message = "Bioauth flow - authenticate transaction is in finalized block",
                            %block_hash,
                        );
                        break;
                    }
                    Some(TransactionStatus::Retracted(block_hash)) => {
                        error!(
                            message = "Bioauth flow - the block this transaction was included in has been retracted",
                            %block_hash,
                        );
                        break;
                    }
                    Some(TransactionStatus::Usurped(_)) => {
                        error!(
                            "Bioauth flow - transaction has been replaced in the pool, by another transaction",
                        );
                        break;
                    }
                    Some(TransactionStatus::Dropped) => {
                        error!(
                            "Bioauth flow - transaction has been dropped from the pool because of the limit",
                        );
                        break;
                    }
                    Some(TransactionStatus::FinalityTimeout(_)) => {
                        error!(
                            "Bioauth flow - maximum number of finality watchers has been reached, old watchers are being removed",
                        );
                        break;
                    }
                    Some(TransactionStatus::Invalid) => {
                        error!(
                            "Bioauth flow - transaction is no longer valid in the current state",
                        );
                        break;
                    }
                    Some(TransactionStatus::Ready) => {
                        info!("Bioauth flow - authenticate transaction is in ready queue")
                    }
                    Some(TransactionStatus::Broadcast(_)) => {
                        info!("Bioauth flow - authenticate transaction is broadcasted");
                    }
                    Some(TransactionStatus::InBlock((block_hash, _))) => {
                        info!(
                            message = "Bioauth flow - authenticate transaction is in block",
                            %block_hash,
                        );
                    }
                    Some(TransactionStatus::Future) => {
                        info!("Bioauth flow - authenticate transaction is in future queue")
                    }
                    None => {
                        error!("Bioauth flow - unexpected transaction flow interruption",);
                        break;
                    }
                }
            }

            info!("Bioauth flow - authenticate transaction complete");
        });

        Ok(tx_hash)
    }

    async fn authenticate_v2(
        &self,
        liveness_data: LivenessData,
    ) -> RpcResult<AuthenticateV2Result> {
        self.deny_unsafe.check_if_safe()?;

        let AuthenticateResponse {
            auth_ticket,
            auth_ticket_signature,
            scan_result_blob,
        } = self
            .do_authenticate(liveness_data)
            .await
            .map_err(AuthenticateV2Error)?;

        info!(message = "We've obtained an auth ticket", auth_ticket = ?auth_ticket);

        Ok(AuthenticateV2Result {
            auth_ticket,
            auth_ticket_signature,
            scan_result_blob,
        })
    }
}
