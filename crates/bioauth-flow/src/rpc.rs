//! RPC interface for the bioauth flow.

use std::marker::PhantomData;
use std::{ops::Deref, sync::Arc};

use futures::channel::oneshot;
use futures::lock::BiLock;
use futures::FutureExt;
use jsonrpc_core::{Error as RpcError, ErrorCode};
use jsonrpc_derive::rpc;
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

use crate::{flow::LivenessDataProvider, Signer};

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

    /// Provide the liveness data for the currently running enrollemnt or authentication process.
    #[rpc(name = "bioauth_provideLivenessData")]
    fn provide_liveness_data(&self, liveness_data: LivenessData) -> FutureResult<()>;

    /// Get the current bioauth status.
    #[rpc(name = "bioauth_status")]
    fn status(&self) -> FutureResult<BioauthStatus<Timestamp>>;
}

/// The shared [`LivenessData`] sender slot, that we can swap with our ephemernal
/// channel upon a liveness data request.
pub type LivenessDataTxSlot = BiLock<Option<oneshot::Sender<LivenessData>>>;

/// Create an linked pair of an empty [`LivenessDataTxSlot`]s.
/// To be used in the initialization process.
pub fn new_liveness_data_tx_slot() -> (LivenessDataTxSlot, LivenessDataTxSlot) {
    BiLock::new(None)
}

/// The RPC implementation.
pub struct Bioauth<
    RobonodeClient,
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
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
    Bioauth<
        RobonodeClient,
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
        liveness_data_tx_slot: Arc<LivenessDataTxSlot>,
        validator_signer: Option<Arc<ValidatorSigner>>,
        client: Arc<Client>,
        pool: Arc<TransactionPool>,
        validator_key_extractor: ValidatorKeyExtractor,
    ) -> Self {
        Self {
            inner: Inner {
                robonode_client,
                liveness_data_tx_slot,
                client,
                pool,
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
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
    Bioauth<
        RobonodeClient,
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
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    > BioauthApi<Timestamp>
    for Bioauth<
        RobonodeClient,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
where
    RobonodeClient: Send + Sync + 'static,
    ValidatorSigner: Send + Sync + 'static,
    TransactionPool: Send + Sync + 'static,
    Client: Send + Sync + 'static,
    ValidatorKeyExtractor: Send + Sync + 'static,
    Timestamp: Send + Sync + 'static,
    ValidatorKeyExtractor::PublicKeyType: Send + Sync,

    RobonodeClient: Deref<Target = robonode_client::Client>,
    ValidatorSigner: Signer<Vec<u8>>,
    <ValidatorSigner as Signer<Vec<u8>>>::Error: std::error::Error + 'static,
    TransactionPool: TransactionPoolT,
    ValidatorKeyExtractor: ValidatorKeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode,
    ValidatorKeyExtractor::PublicKeyType: AsRef<[u8]>,
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
    /// See [`Inner::get_facetec_device_sdk_params`].
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams> {
        self.with_inner_clone(move |inner| inner.get_facetec_device_sdk_params())
    }

    /// See [`Inner::get_facetec_session_token`].
    fn get_facetec_session_token(&self) -> FutureResult<String> {
        self.with_inner_clone(move |inner| inner.get_facetec_session_token())
    }

    /// See [`Inner::enroll`].
    fn enroll(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.enroll(liveness_data))
    }

    /// See [`Inner::authenticate`].
    fn authenticate(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.authenticate(liveness_data))
    }

    /// See [`Inner::provide_liveness_data`].
    fn provide_liveness_data(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.provide_liveness_data(liveness_data))
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
    ValidatorSigner,
    Client,
    TransactionPool,
    Timestamp,
    ValidatorKeyExtractor,
> {
    /// The robonode client, used for fetching the FaceTec Session Token.
    robonode_client: RobonodeClient,
    /// The liveness data provider sink.
    /// We need an [`Arc`] here to allow sharing the data from across multiple invocations of the
    /// RPC extension builder that will be using this RPC.
    liveness_data_tx_slot: Arc<LivenessDataTxSlot>,
    /// The client to use for transactions.
    client: Arc<Client>,
    /// The transaction pool to use.
    pool: Arc<TransactionPool>,
    /// The type that provides signing with the validator private key.
    validator_signer: Option<Arc<ValidatorSigner>>,
    /// Provider of the local validator key.
    validator_key_extractor: ValidatorKeyExtractor,
    /// The phantom types.
    phantom: PhantomData<Timestamp>,
}

impl<
        RobonodeClient,
        ValidatorSigner,
        Client,
        TransactionPool,
        Timestamp,
        ValidatorKeyExtractor,
    >
    Inner<
        RobonodeClient,
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
    TransactionPool: sc_transaction_pool_api::TransactionPool,
    ValidatorKeyExtractor: ValidatorKeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode + AsRef<[u8]>,
    ValidatorKeyExtractor::PublicKeyType: AsRef<[u8]>,
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

        let at = self.client.info().best_hash;

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

        let public_key = self.validator_public_key()?.ok_or(RpcError {
                code: ErrorCode::ServerError(1),
                message: "Validator public key not found".to_string(),
                data: None,
        })?;
        self.robonode_client
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
                public_key: public_key.as_ref(),
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

     /// Collect the liveness data and provide to the consumer.
    async fn provide_liveness_data(self: Arc<Self>, liveness_data: LivenessData) -> Result<()> {
        let maybe_tx = {
            let mut maybe_tx_guard = self.liveness_data_tx_slot.lock().await;
            maybe_tx_guard.take() // take the guarded option value and release the lock asap
        };
        let tx = maybe_tx.ok_or_else(|| RpcError {
            code: ErrorCode::InternalError,
            message: "Flow is not engaged, unable to accept liveness data".into(),
            data: None,
        })?;
        tx.send(liveness_data).map_err(|_| RpcError {
            code: ErrorCode::InternalError,
            message: "Flow was aborted before the liveness data could be submitted".into(),
            data: None,
        })?;
        Ok(())
    }

    /// Obtain the status of the bioauth.
    async fn status(self: Arc<Self>) -> Result<BioauthStatus<Timestamp>> {
        let own_key = self.validator_public_key()?;
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

    /// Extract the validator public key.
    fn validator_public_key(&self) -> Result<Option<ValidatorKeyExtractor::PublicKeyType>>  {
         self
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
            })
    }

    /// Return the opaque liveness data and corresponding signature.
    async fn sign(&self, liveness_data: &LivenessData) -> Result<(OpaqueLivenessData, Vec<u8>)> {
        let opaque_liveness_data = OpaqueLivenessData::from(liveness_data);

        let signature = self
            .validator_signer
            .as_ref()
            .ok_or(RpcError {
                code: ErrorCode::InternalError,
                message: "Validator signer not found".into(),
                data: None,
            })?
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

/// Provider implements a [`LivenessDataProvider`].
pub struct Provider {
    /// The shared liveness data sender slot, that we can swap with our ephemernal
    /// channel upon a liveness data reuqest.
    liveness_data_tx_slot: LivenessDataTxSlot,
}

impl Provider {
    /// Construct a new [`Provider`].
    pub fn new(liveness_data_tx_slot: LivenessDataTxSlot) -> Self {
        Self {
            liveness_data_tx_slot,
        }
    }
}

#[async_trait::async_trait]
impl LivenessDataProvider for Provider {
    type Error = oneshot::Canceled;

    async fn provide(&mut self) -> std::result::Result<LivenessData, Self::Error> {
        let (tx, rx) = oneshot::channel();

        {
            let mut maybe_tx_guard = self.liveness_data_tx_slot.lock().await;
            let _ = maybe_tx_guard.insert(tx); // insert a new sender value and free the lock asap
        }

        Ok(rx.await?)
    }
}
