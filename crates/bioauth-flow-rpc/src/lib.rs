//! The bioauth flow RPC implementation.
//!
//! It is the logic of communication between the humanode (aka humanode-peer),
//! the app on the handheld device that performs the biometric capture,
//! and the robonode server that issues auth tickets.

use std::marker::PhantomData;
use std::sync::Arc;

use bioauth_flow_api::BioauthFlowApi;
use bioauth_keys::traits::KeyExtractor as KeyExtractorT;
use errors::{
    AuthenticateError, BioauthTxError, EnrollError, GetFacetecDeviceSdkParamsError,
    GetFacetecSessionToken, RobonodeError, SignerError, StatusError,
};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use robonode_client::{AuthenticateRequest, EnrollRequest};
use rpc_deny_unsafe::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool as TransactionPoolT;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sp_api::{BlockT, Decode, Encode, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use tracing::*;

mod errors;

/// Signer provides signatures for the data.
#[async_trait::async_trait]
pub trait Signer<S> {
    /// Signature error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Sign the provided data and return the signature, or an error if the signing fails.
    async fn sign<'a, D>(&self, data: D) -> std::result::Result<S, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// A factory that spits out [`Signer`]s.
pub trait SignerFactory<S, K> {
    /// The type of [`Signer`] this factory will create.
    type Signer: Signer<S>;

    /// Create a [`Signer`] using the provided public key.
    fn new_signer(&self, key: K) -> Self::Signer;
}

impl<S, T, F, K, P> SignerFactory<T, K> for P
where
    P: std::ops::Deref<Target = F>,
    F: Fn(K) -> S,
    S: Signer<T>,
{
    type Signer = S;

    fn new_signer(&self, key: K) -> Self::Signer {
        self(key)
    }
}

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
pub trait Bioauth<Timestamp> {
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

    /// Authenticate with provided liveness data.
    #[method(name = "bioauth_authenticate")]
    async fn authenticate(&self, liveness_data: LivenessData) -> RpcResult<()>;
}

/// The RPC implementation.
pub struct Bioauth<
    RobonodeClient,
    ValidatorKeyExtractor,
    ValidatorSignerFactory,
    Client,
    Block,
    Timestamp,
    TransactionPool,
> {
    /// The robonode client, used for fetching the FaceTec Session Token.
    robonode_client: RobonodeClient,
    /// Provider of the local validator key.
    validator_key_extractor: ValidatorKeyExtractor,
    /// The type that provides signing with the validator private key.
    validator_signer_factory: ValidatorSignerFactory,
    /// The substrate client, provides access to the runtime APIs.
    client: Arc<Client>,
    /// The transaction pool to use.
    pool: Arc<TransactionPool>,
    /// Whether to deny unsafe calls or not.
    deny_unsafe: DenyUnsafe,
    /// The phantom types.
    phantom_types: PhantomData<(Block, Timestamp)>,
}

impl<
        RobonodeClient,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Client,
        Block,
        Timestamp,
        TransactionPool,
    >
    Bioauth<
        RobonodeClient,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Client,
        Block,
        Timestamp,
        TransactionPool,
    >
{
    /// Create a new [`Bioauth`] API implementation.
    pub fn new(
        robonode_client: RobonodeClient,
        validator_key_extractor: ValidatorKeyExtractor,
        validator_signer_factory: ValidatorSignerFactory,
        client: Arc<Client>,
        pool: Arc<TransactionPool>,
        deny_unsafe: DenyUnsafe,
    ) -> Self {
        Self {
            robonode_client,
            validator_key_extractor,
            validator_signer_factory,
            client,
            pool,
            deny_unsafe,
            phantom_types: PhantomData,
        }
    }
}

impl<
        RobonodeClient,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Client,
        Block,
        Timestamp,
        TransactionPool,
    >
    Bioauth<
        RobonodeClient,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Client,
        Block,
        Timestamp,
        TransactionPool,
    >
where
    ValidatorKeyExtractor: KeyExtractorT,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    ValidatorSignerFactory: SignerFactory<Vec<u8>, ValidatorKeyExtractor::PublicKeyType>,
    <<ValidatorSignerFactory as SignerFactory<Vec<u8>, ValidatorKeyExtractor::PublicKeyType>>::Signer as Signer<Vec<u8>>>::Error:
        std::error::Error + 'static,
{
    /// Return the opaque liveness data and corresponding signature.
    async fn sign(&self, liveness_data: &LivenessData) -> RpcResult<(OpaqueLivenessData, Vec<u8>)> {
        let opaque_liveness_data = OpaqueLivenessData::from(liveness_data);
        let validator_key = rpc_validator_key_logic::validator_public_key(&self.validator_key_extractor)?;
        let signer = self.validator_signer_factory.new_signer(validator_key);

        let signature = signer.sign(&opaque_liveness_data).await.map_err(|error| {
            tracing::error!(message = "Signing failed", ?error);
            SignerError::SigningFailed
        })?;

        Ok((opaque_liveness_data, signature))
    }
}

#[jsonrpsee::core::async_trait]
impl<
        RobonodeClient,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Client,
        Block,
        Timestamp,
        TransactionPool,
    > BioauthServer<Timestamp>
    for Bioauth<
        RobonodeClient,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Client,
        Block,
        Timestamp,
        TransactionPool,
    >
where
    RobonodeClient: Send + Sync + 'static,
    ValidatorKeyExtractor: Send + Sync + 'static,
    ValidatorKeyExtractor::PublicKeyType: Send + Sync + 'static,
    ValidatorSignerFactory: Send + Sync + 'static,
    ValidatorSignerFactory::Signer: Send + Sync + 'static,
    Client: Send + Sync + 'static,
    Block: Send + Sync + 'static,
    Timestamp: Send + Sync + 'static,
    TransactionPool: Send + Sync + 'static,

    RobonodeClient: AsRef<robonode_client::Client>,
    ValidatorKeyExtractor: KeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode + AsRef<[u8]>,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    ValidatorSignerFactory: SignerFactory<Vec<u8>, ValidatorKeyExtractor::PublicKeyType>,
    <<ValidatorSignerFactory as SignerFactory<Vec<u8>, ValidatorKeyExtractor::PublicKeyType>>::Signer as Signer<Vec<u8>>>::Error:
        std::error::Error + 'static,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client: Send + Sync + 'static,
    Client::Api:
        bioauth_flow_api::BioauthFlowApi<Block, ValidatorKeyExtractor::PublicKeyType, Timestamp>,
    Block: BlockT,
    Timestamp: Encode + Decode,
    TransactionPool: TransactionPoolT<Block = Block>,
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

    async fn status(&self) -> RpcResult<BioauthStatus<Timestamp>> {
        let own_key = match rpc_validator_key_logic::validator_public_key(&self.validator_key_extractor) {
            Ok(v) => v,
            Err(rpc_validator_key_logic::ValidatorKeyError::MissingValidatorKey) => return Ok(BioauthStatus::Unknown),
            Err(err) => return Err(StatusError::KeyExtraction(err).into()),
        };

        // Extract an id of the last imported block.
        let at = sp_api::BlockId::Hash(self.client.info().best_hash);

        let status = self
            .client
            .runtime_api()
            .bioauth_status(&at, &own_key)
            .map_err(StatusError::RuntimeApi)?;

        Ok(status.into())
    }

    async fn enroll(&self, liveness_data: LivenessData) -> RpcResult<()> {
        self.deny_unsafe.check_if_safe()?;

        info!("Bioauth flow - enrolling in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await
            .map_err(|_| EnrollError::Signer(SignerError::SigningFailed))?;

        let public_key = rpc_validator_key_logic::validator_public_key(&self.validator_key_extractor)
            .map_err(EnrollError::KeyExtraction)?;
        self.robonode_client
            .as_ref()
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
                public_key: public_key.as_ref(),
            })
            .await
            .map_err(|err| {
                match err {
                    robonode_client::Error::Call(
                        robonode_client::EnrollError::FaceScanRejected
                    ) => EnrollError::Robonode(RobonodeError::ShouldRetry),
                    _ => EnrollError::Robonode(RobonodeError::Other(err.to_string())),
                }
            })?;

        info!("Bioauth flow - enrolling complete");

        Ok(())
    }

    async fn authenticate(&self, liveness_data: LivenessData) -> RpcResult<()> {
        self.deny_unsafe.check_if_safe()?;

        info!("Bioauth flow - authentication in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await
            .map_err(|_| AuthenticateError::Signer(SignerError::SigningFailed))?;

        let response = self
            .robonode_client
            .as_ref()
            .authenticate(AuthenticateRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await
            .map_err(|err| {
                match err {
                    robonode_client::Error::Call(
                        robonode_client::AuthenticateError::FaceScanRejected
                    ) => AuthenticateError::Robonode(RobonodeError::ShouldRetry),
                    _ => AuthenticateError::Robonode(RobonodeError::Other(err.to_string())),
                }
            })?;

        info!("Bioauth flow - authentication complete");

        info!(message = "We've obtained an auth ticket", auth_ticket = ?response.auth_ticket);

        let at = sp_api::BlockId::Hash(self.client.info().best_hash);

        let ext = self
            .client
            .runtime_api()
            .create_authenticate_extrinsic(
                &at,
                response.auth_ticket.into(),
                response.auth_ticket_signature.into(),
            )
            .map_err(AuthenticateError::RuntimeApi)?;

        self.pool
            .submit_and_watch(
                &at,
                sp_runtime::transaction_validity::TransactionSource::Local,
                ext,
            )
            .await
            .map_err(|err| AuthenticateError::TxPool(BioauthTxError::from(err)))?;

        info!("Bioauth flow - authenticate transaction complete");

        Ok(())
    }
}
