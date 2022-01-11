//! The bioauth flow implementation, aka the logic for communication between the humanode
//! (aka humanode-peer), the app on the handheld device that perform that biometric capture,
//! and the robonode server that's responsible for authenticating against the bioauth system.

use std::marker::PhantomData;
use std::{ops::Deref, sync::Arc};

use futures::FutureExt;
use jsonrpc_core::{Error as RpcError, ErrorCode};
use jsonrpc_derive::rpc;
use sc_client_api::UsageProvider;
use sc_transaction_pool_api::TransactionPool as TransactionPoolT;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sp_api::{Decode, Encode, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use tracing::*;

pub use bioauth_consensus::ValidatorKeyExtractor as ValidatorKeyExtractorT;
use bioauth_flow_api::BioauthFlowApi;
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use robonode_client::{AuthenticateRequest, EnrollRequest};

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

/// A result type that wraps.
pub type Result<T> = std::result::Result<T, RpcError>;

/// A futures that resolves to the specified `T`, or an [`RpcError`].
pub type FutureResult<T> = jsonrpc_core::BoxFuture<Result<T>>;

/// The parameters necessary to initialize the FaceTec Device SDK.
pub type FacetecDeviceSdkParams = Map<String, Value>;

/// The API exposed via JSON-RPC.
#[rpc]
pub trait BioauthApi<Timestamp> {
    /// Get the configuration required for the Device SDK.
    #[rpc(name = "bioauth_getFacetecDeviceSdkParams")]
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams>;

    /// Get a FaceTec Session Token.
    #[rpc(name = "bioauth_getFacetecSessionToken")]
    fn get_facetec_session_token(&self) -> FutureResult<String>;

    /// Enroll with provided liveness data.
    #[rpc(name = "bioauth_enroll")]
    fn enroll(&self, liveness_data: LivenessData) -> FutureResult<()>;

    /// Authenticate with provided liveness data.
    #[rpc(name = "bioauth_authenticate")]
    fn authenticate(&self, liveness_data: LivenessData) -> FutureResult<()>;

    /// Get the current bioauth status.
    #[rpc(name = "bioauth_status")]
    fn status(&self) -> FutureResult<BioauthStatus<Timestamp>>;
}

/// The RPC implementation.
pub struct Bioauth<
    RobonodeClient,
    ValidatorPublicKey,
    ValidatorSigner,
    Client,
    TransactionPool,
    Timestamp,
    ValidatorKeyExtractor,
> {
    /// The underlying implementation.
    /// We have to wrap it with `Arc` because `jsonrpc-core` doesn't allow us to use `self` for
    /// the duration of the future; we `clone` the `Arc` to get the `'static` lifetime to
    /// a shared `Inner` instead.
    inner: Arc<
        Inner<
            RobonodeClient,
            ValidatorPublicKey,
            ValidatorSigner,
            Client,
            TransactionPool,
            Timestamp,
            ValidatorKeyExtractor,
        >,
    >,
}

impl<
        RobonodeClient,
        ValidatorPublicKey,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
    Bioauth<
        RobonodeClient,
        ValidatorPublicKey,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
{
    /// Create a new [`Bioauth`] API implementation.
    pub fn new(
        robonode_client: RobonodeClient,
        validator_public_key: Arc<ValidatorPublicKey>,
        validator_signer: Arc<ValidatorSigner>,
        client: Arc<Client>,
        pool: Arc<TransactionPool>,
        validator_key_extractor: ValidatorKeyExtractor,
    ) -> Self {
        Self {
            inner: Inner {
                robonode_client,
                client,
                pool,
                validator_public_key,
                validator_signer,
                validator_key_extractor,
                phantom: Default::default(),
            }
            .into(),
        }
    }
}

impl<
        RobonodeClient,
        ValidatorPublicKey,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
    Bioauth<
        RobonodeClient,
        ValidatorPublicKey,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
{
    /// A helper function that provides a convenient way to execute a future with a clone of
    /// the `Arc<Inner>`.
    /// It also boxes the resulting [`Future`] `Fut` so it fits into the [`FutureResult`].
    fn with_inner_clone<F, Fut, R>(&self, f: F) -> FutureResult<R>
    where
        F: FnOnce(
            Arc<
                Inner<
                    RobonodeClient,
                    ValidatorPublicKey,
                    ValidatorSigner,
                    Client,
                    TransactionPool,
                    Timestamp,
                    ValidatorKeyExtractor,
                >,
            >,
        ) -> Fut,
        Fut: std::future::Future<Output = Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        let inner = Arc::clone(&self.inner);
        f(inner).boxed()
    }
}

impl<
        RobonodeClient,
        ValidatorPublicKey,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    > BioauthApi<Timestamp>
    for Bioauth<
        RobonodeClient,
        ValidatorPublicKey,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
where
    RobonodeClient: Send + Sync + 'static,
    ValidatorSigner: Send + Sync + 'static,
    ValidatorPublicKey: Send + Sync + 'static,
    TransactionPool: Send + Sync + 'static,
    Client: Send + Sync + 'static,
    ValidatorKeyExtractor: Send + Sync + 'static,
    Timestamp: Send + Sync + 'static,

    RobonodeClient: Deref<Target = robonode_client::Client>,
    ValidatorSigner: Signer<Vec<u8>>,
    <ValidatorSigner as Signer<Vec<u8>>>::Error: std::error::Error + 'static,
    ValidatorPublicKey: AsRef<[u8]>,
    TransactionPool: TransactionPoolT,
    Client: UsageProvider<<TransactionPool as TransactionPoolT>::Block>,
    ValidatorKeyExtractor: ValidatorKeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    <<TransactionPool as TransactionPoolT>::Block as sp_runtime::traits::Block>::Extrinsic:
        From<humanode_runtime::UncheckedExtrinsic>,
    Client: HeaderBackend<TransactionPool::Block>,
    Client: ProvideRuntimeApi<TransactionPool::Block>,
    Client::Api: bioauth_flow_api::BioauthFlowApi<
        TransactionPool::Block,
        ValidatorKeyExtractor::PublicKeyType,
        Timestamp,
    >,
    Timestamp: Encode + Decode,
{
    /// See `Inner::get_facetec_device_sdk_params`.
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams> {
        self.with_inner_clone(move |inner| inner.get_facetec_device_sdk_params())
    }

    /// See `Inner::get_facetec_session_token`.
    fn get_facetec_session_token(&self) -> FutureResult<String> {
        self.with_inner_clone(move |inner| inner.get_facetec_session_token())
    }

    /// See `Inner::enroll`.
    fn enroll(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.enroll(liveness_data))
    }

    /// See `Inner::authenticate`.
    fn authenticate(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.authenticate(liveness_data))
    }

    /// See [`Inner::status`].
    fn status(&self) -> FutureResult<BioauthStatus<Timestamp>> {
        self.with_inner_clone(move |inner| inner.status())
    }
}

/// The underlying implementation of the RPC part, extracted into a subobject to work around
/// the common pitfall with the poor async engines implementations of requiring future objects to
/// be static.
/// Stop it people, why do you even use Rust if you do things like this? Ffs...
/// See https://github.com/paritytech/jsonrpc/issues/580
struct Inner<
    RobonodeClient,
    ValidatorPublicKey,
    ValidatorSigner,
    Client,
    TransactionPool,
    Timestamp,
    ValidatorKeyExtractor,
> {
    /// The robonode client, used for fetching the FaceTec Session Token.
    robonode_client: RobonodeClient,
    /// The client to use for transactions.
    client: Arc<Client>,
    /// The transaction pool to use.
    pool: Arc<TransactionPool>,
    /// The type used to encode the public key.
    validator_public_key: Arc<ValidatorPublicKey>,
    /// The type that provides signing with the validator private key.
    validator_signer: Arc<ValidatorSigner>,
    /// Provider of the local validator key.
    validator_key_extractor: ValidatorKeyExtractor,
    /// The phantom types.
    phantom: PhantomData<Timestamp>,
}

impl<
        RobonodeClient,
        ValidatorPublicKey,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
    Inner<
        RobonodeClient,
        ValidatorPublicKey,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
where
    RobonodeClient: Deref<Target = robonode_client::Client>,
    ValidatorSigner: Signer<Vec<u8>>,
    <ValidatorSigner as Signer<Vec<u8>>>::Error: std::error::Error + 'static,
    ValidatorPublicKey: AsRef<[u8]>,
    TransactionPool: sc_transaction_pool_api::TransactionPool,
    Client: UsageProvider<<TransactionPool as sc_transaction_pool_api::TransactionPool>::Block>,
    ValidatorKeyExtractor: ValidatorKeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    <<TransactionPool as sc_transaction_pool_api::TransactionPool>::Block as sp_runtime::traits::Block>::Extrinsic:
        From<humanode_runtime::UncheckedExtrinsic>,
    Client: HeaderBackend<TransactionPool::Block>,
    Client: ProvideRuntimeApi<TransactionPool::Block>,
    Client::Api:
        bioauth_flow_api::BioauthFlowApi<TransactionPool::Block, ValidatorKeyExtractor::PublicKeyType, Timestamp>,
    Timestamp: Encode + Decode,
{
    /// Get the FaceTec Device SDK parameters to use at the device.
    async fn get_facetec_device_sdk_params(self: Arc<Self>) -> Result<FacetecDeviceSdkParams> {
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

    /// Get the FaceTec Session Token.
    async fn get_facetec_session_token(self: Arc<Self>) -> Result<String> {
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

    /// Submit an authenticate request to robonode with liveness data, followed by an authenticate
    /// transaction to chain.
    async fn authenticate(self: Arc<Self>, liveness_data: LivenessData) -> Result<()> {
        info!("Bioauth flow - authentication in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await?;

        let response = self
            .robonode_client
            .authenticate(AuthenticateRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;

        info!("Bioauth flow - authentication complete");

        info!(message = "We've obtained an auth ticket", auth_ticket = ?response.auth_ticket);

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
            .map_err(|_| RpcError {
                code: ErrorCode::ServerError(1),
                message: "Transaction failed".to_string(),
                data: None,
            })?;

        info!("Bioauth flow - authenticate transaction complete");

        Ok(())
    }

    /// Submit an enroll request to robonode with provided liveness data.
    async fn enroll(self: Arc<Self>, liveness_data: LivenessData) -> Result<()> {
        info!("Bioauth flow - enrolling in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await?;

        self.robonode_client
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
                public_key: self.validator_public_key.as_ref().as_ref(),
            })
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;

        info!("Bioauth flow - enrolling complete");

        Ok(())
    }

    /// Obtain the status of the bioauth.
    async fn status(self: Arc<Self>) -> Result<BioauthStatus<Timestamp>> {
        let own_key = self
            .validator_key_extractor
            .extract_validator_key()
            .map_err(|error| {
                tracing::error!(
                    message = "Unable to extract own key at bioauth flow RPC",
                    ?error
                );
                RpcError {
                    code: ErrorCode::InternalError,
                    message: "Unable to extract own key".into(),
                    data: None,
                }
            })?;
        let own_key = match own_key {
            Some(v) => v,
            None => return Ok(BioauthStatus::Unknown),
        };

        // Extract an id of the last imported block.
        let at = sp_api::BlockId::Hash(self.client.info().best_hash);

        let status = self
            .client
            .runtime_api()
            .bioauth_status(&at, &own_key)
            .map_err(|err| RpcError {
                code: ErrorCode::InternalError,
                message: format!("Unable to get status from the runtime: {}", err),
                data: None,
            })?;

        Ok(status.into())
    }

    /// Return the opaque liveness data and corresponding signature.
    async fn sign(&self, liveness_data: &LivenessData) -> Result<(OpaqueLivenessData, Vec<u8>)> {
        let opaque_liveness_data = OpaqueLivenessData::from(liveness_data);

        let signature = self
            .validator_signer
            .sign(&opaque_liveness_data)
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("Signing failed: {}", err),
                data: None,
            })?;

        Ok((opaque_liveness_data, signature))
    }
}
