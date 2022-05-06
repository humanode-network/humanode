//! RPC interface for the bioauth flow.

use std::marker::PhantomData;
use std::sync::Arc;

pub use bioauth_consensus::ValidatorKeyExtractor as ValidatorKeyExtractorT;
use bioauth_flow_api::BioauthFlowApi;
use futures::channel::oneshot;
use futures::lock::BiLock;
use futures::FutureExt;
use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode as RpcErrorCode;
use jsonrpc_derive::rpc;
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use robonode_client::{AuthenticateRequest, EnrollRequest};
use rotate_keys_api::RotateKeysApi;
use sc_transaction_pool_api::TransactionPool as TransactionPoolT;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sp_api::{BlockT, Decode, Encode, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::transaction_validity::InvalidTransaction;
use sp_session::SessionKeys;
use tracing::*;

use crate::{flow::LivenessDataProvider, Signer, SignerFactory};

/// A result type that wraps.
pub type Result<T> = std::result::Result<T, RpcError>;

/// A futures that resolves to the specified `T`, or an [`RpcError`].
pub type FutureResult<T> = jsonrpc_core::BoxFuture<Result<T>>;

/// The parameters necessary to initialize the FaceTec Device SDK.
type FacetecDeviceSdkParams = Map<String, Value>;

/// The RPC error context we provide to trigger the face capture logic again,
/// effectively requesting a retry of the same request with a new liveness data.
#[derive(Debug)]
struct ShouldRetry;

impl From<ShouldRetry> for Value {
    fn from(_: ShouldRetry) -> Self {
        serde_json::json!({ "shouldRetry": true })
    }
}

/// The RPC error context we provide to trigger the face capture logic again,
/// effectively requesting a retry of the same request with a new liveness data.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionPoolErrorDetails {
    /// The error kind.
    kind: TransactionPoolErrorKind,
    /// The message from the inner transaction pool error.
    inner_error: String,
}

/// The error kinds that we expose in the RPC that originate from the transaction pool.
#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum TransactionPoolErrorKind {
    /// Auth ticket signature was not valid.
    AuthTicketSignatureInvalid,
    /// We were unable to parse the auth ticket (although its signature was supposed to be
    /// validated by now).
    UnableToParseAuthTicket,
    /// The nonce was already seen by the system.
    NonceAlreadyUsed,
    /// The aactive authentication issued by this ticket is still on.
    AlreadyAuthenticated,
}

impl From<TransactionPoolErrorDetails> for Value {
    fn from(val: TransactionPoolErrorDetails) -> Self {
        serde_json::json!({ "transactionPoolErrorDetails": val })
    }
}

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
enum ErrorCode {
    /// Signer has failed.
    Signer = 100,
    /// Request to robonode has failed.
    Robonode = 200,
    /// Call to runtime api has failed.
    RuntimeApi = 300,
    /// Authenticate transaction has failed.
    Transaction = 400,
    /// Validator key is not available.
    MissingValidatorKey = 500,
    /// Validator key extraction has failed.
    ValidatorKeyExtraction = 600,
    /// Liveness data was not provided.
    MissingLivenessData = 700,
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

/// The API exposed via JSON-RPC.
#[rpc]
pub trait BioauthApi<Timestamp> {
    /// Get the configuration required for the Device SDK.
    #[rpc(name = "bioauth_getFacetecDeviceSdkParams")]
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams>;

    /// Get a FaceTec Session Token.
    #[rpc(name = "bioauth_getFacetecSessionToken")]
    fn get_facetec_session_token(&self) -> FutureResult<String>;

    /// Provide the liveness data for the currently running enrollemnt or authentication process.
    #[rpc(name = "bioauth_provideLivenessData")]
    fn provide_liveness_data(&self, liveness_data: LivenessData) -> FutureResult<()>;

    /// Get the current bioauth status.
    #[rpc(name = "bioauth_status")]
    fn status(&self) -> FutureResult<BioauthStatus<Timestamp>>;

    /// Enroll with provided liveness data.
    #[rpc(name = "bioauth_enroll")]
    fn enroll(&self, liveness_data: LivenessData) -> FutureResult<()>;

    /// Authenticate with provided liveness data.
    #[rpc(name = "bioauth_authenticate")]
    fn authenticate(&self, liveness_data: LivenessData) -> FutureResult<()>;
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
    ValidatorKeyExtractor,
    ValidatorSignerFactory,
    Client,
    Block,
    Timestamp,
    TransactionPool,
> {
    /// The underlying implementation.
    /// We have to wrap it with `Arc` because `jsonrpc-core` doesn't allow us to use `self` for
    /// the duration of the future; we `clone` the `Arc` to get the `'static` lifetime to
    /// a shared `Inner` instead.
    inner: Arc<
        Inner<
            RobonodeClient,
            ValidatorKeyExtractor,
            ValidatorSignerFactory,
            Client,
            Block,
            Timestamp,
            TransactionPool,
        >,
    >,
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
        liveness_data_tx_slot: Arc<LivenessDataTxSlot>,
        validator_key_extractor: ValidatorKeyExtractor,
        validator_signer_factory: ValidatorSignerFactory,
        client: Arc<Client>,
        pool: Arc<TransactionPool>,
    ) -> Self {
        let inner = Inner {
            robonode_client,
            liveness_data_tx_slot,
            validator_key_extractor,
            validator_signer_factory,
            client,
            pool,
            phantom_types: PhantomData,
        };
        Self {
            inner: Arc::new(inner),
        }
    }

    /// A helper function that provides a convenient way to execute a future with a clone of
    /// the `Arc<Inner>`.
    /// It also boxes the resulting [`Future`] `Fut` so it fits into the [`FutureResult`].
    fn with_inner_clone<F, Fut, R>(&self, f: F) -> FutureResult<R>
    where
        F: FnOnce(
            Arc<
                Inner<
                    RobonodeClient,
                    ValidatorKeyExtractor,
                    ValidatorSignerFactory,
                    Client,
                    Block,
                    Timestamp,
                    TransactionPool,
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
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Client,
        Block,
        Timestamp,
        TransactionPool,
    > BioauthApi<Timestamp>
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
    ValidatorKeyExtractor: ValidatorKeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode + AsRef<[u8]>,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    ValidatorSignerFactory: SignerFactory<Vec<u8>, ValidatorKeyExtractor::PublicKeyType>,
    <<ValidatorSignerFactory as SignerFactory<Vec<u8>, ValidatorKeyExtractor::PublicKeyType>>::Signer as Signer<Vec<u8>>>::Error:
        std::error::Error + 'static,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client: Send + Sync + 'static,
    Client::Api: SessionKeys<Block>,
    Client::Api:
        bioauth_flow_api::BioauthFlowApi<Block, ValidatorKeyExtractor::PublicKeyType, Timestamp>,
    Client::Api: RotateKeysApi<Block, ValidatorKeyExtractor::PublicKeyType>,
    Block: BlockT,
    Timestamp: Encode + Decode,
    TransactionPool: TransactionPoolT<Block = Block>,
{
    /// See [`Inner::get_facetec_device_sdk_params`].
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams> {
        self.with_inner_clone(move |inner| inner.get_facetec_device_sdk_params())
    }

    /// See [`Inner::get_facetec_session_token`].
    fn get_facetec_session_token(&self) -> FutureResult<String> {
        self.with_inner_clone(move |inner| inner.get_facetec_session_token())
    }

    /// See [`Inner::provide_liveness_data`].
    fn provide_liveness_data(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.provide_liveness_data(liveness_data))
    }

    /// See [`Inner::status`].
    fn status(&self) -> FutureResult<BioauthStatus<Timestamp>> {
        self.with_inner_clone(move |inner| inner.status())
    }

    /// See [`Inner::enroll`].
    fn enroll(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.enroll(liveness_data))
    }

    /// See [`Inner::authenticate`].
    fn authenticate(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.authenticate(liveness_data))
    }
}

/// The underlying implementation of the RPC part, extracted into a subobject to work around
/// the common pitfall with the poor async engines implementations of requiring future objects to
/// be static.
/// Stop it people, why do you even use Rust if you do things like this? Ffs...
/// See https://github.com/paritytech/jsonrpc/issues/580
struct Inner<
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
    /// The liveness data provider sink.
    /// We need an [`Arc`] here to allow sharing the data from across multiple invocations of the
    /// RPC extension builder that will be using this RPC.
    liveness_data_tx_slot: Arc<LivenessDataTxSlot>,
    /// Provider of the local validator key.
    validator_key_extractor: ValidatorKeyExtractor,
    /// The type that provides signing with the validator private key.
    validator_signer_factory: ValidatorSignerFactory,
    /// The substrate client, provides access to the runtime APIs.
    client: Arc<Client>,
    /// The transaction pool to use.
    pool: Arc<TransactionPool>,
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
    Inner<
        RobonodeClient,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Client,
        Block,
        Timestamp,
        TransactionPool,
    >
where
    RobonodeClient: AsRef<robonode_client::Client>,
    ValidatorKeyExtractor: ValidatorKeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode + AsRef<[u8]>,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    ValidatorSignerFactory: SignerFactory<Vec<u8>, ValidatorKeyExtractor::PublicKeyType>,
    <<ValidatorSignerFactory as SignerFactory<Vec<u8>, ValidatorKeyExtractor::PublicKeyType>>::Signer as Signer<Vec<u8>>>::Error:
        std::error::Error + 'static,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client: Send + Sync + 'static,
    Client::Api: SessionKeys<Block>,
    Client::Api:
        bioauth_flow_api::BioauthFlowApi<Block, ValidatorKeyExtractor::PublicKeyType, Timestamp>,
    Client::Api: RotateKeysApi<Block, ValidatorKeyExtractor::PublicKeyType>,
    Block: BlockT,
    Timestamp: Encode + Decode,
    TransactionPool: TransactionPoolT<Block = Block>,
{
    /// Get the FaceTec Device SDK parameters to use at the device.
    async fn get_facetec_device_sdk_params(self: Arc<Self>) -> Result<FacetecDeviceSdkParams> {
        let res = self
            .robonode_client
            .as_ref()
            .get_facetec_device_sdk_params()
            .await
            .map_err(|err| RpcError {
                code: RpcErrorCode::ServerError(ErrorCode::Robonode as _),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;
        Ok(res)
    }

    /// Get the FaceTec Session Token.
    async fn get_facetec_session_token(self: Arc<Self>) -> Result<String> {
        let res = self
            .robonode_client
            .as_ref()
            .get_facetec_session_token()
            .await
            .map_err(|err| RpcError {
                code: RpcErrorCode::ServerError(ErrorCode::Robonode as _),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;
        Ok(res.session_token)
    }

    /// Collect the liveness data and provide to the consumer.
    async fn provide_liveness_data(self: Arc<Self>, liveness_data: LivenessData) -> Result<()> {
        let maybe_tx = {
            let mut maybe_tx_guard = self.liveness_data_tx_slot.lock().await;
            maybe_tx_guard.take() // take the guarded option value and release the lock asap
        };
        let tx = maybe_tx.ok_or_else(|| RpcError {
            code: RpcErrorCode::ServerError(ErrorCode::MissingLivenessData as _),
            message: "Flow is not engaged, unable to accept liveness data".into(),
            data: None,
        })?;
        tx.send(liveness_data).map_err(|_| RpcError {
            code: RpcErrorCode::ServerError(ErrorCode::MissingLivenessData as _),
            message: "Flow was aborted before the liveness data could be submitted".into(),
            data: None,
        })?;
        Ok(())
    }

    /// Obtain the status of the bioauth.
    async fn status(self: Arc<Self>) -> Result<BioauthStatus<Timestamp>> {
        let own_key = match self.validator_public_key()? {
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
                code: RpcErrorCode::ServerError(ErrorCode::RuntimeApi as _),
                message: format!("Unable to get status from the runtime: {}", err),
                data: None,
            })?;

        Ok(status.into())
    }

    /// Submit an enroll request to robonode with provided liveness data.
    async fn enroll(self: Arc<Self>, liveness_data: LivenessData) -> Result<()> {
        info!("Bioauth flow - enrolling in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await?;

        let public_key = self.validator_public_key()?.ok_or(RpcError {
            code: RpcErrorCode::ServerError(ErrorCode::MissingValidatorKey as _),
            message: "Validator key not available".to_string(),
            data: None,
        })?;
        self.robonode_client
            .as_ref()
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
                public_key: public_key.as_ref(),
            })
            .await
            .map_err(|err| {
                let data = match err {
                    robonode_client::Error::Call(
                        robonode_client::EnrollError::FaceScanRejected
                    ) => Some(ShouldRetry.into()),
                    _ => None,
                };

                RpcError {
                    code: RpcErrorCode::ServerError(ErrorCode::Robonode as _),
                    message: format!("request to the robonode failed: {}", err),
                    data,
                }
            })?;

        info!("Bioauth flow - enrolling complete");

        Ok(())
    }

    /// Submit an authenticate request to robonode with liveness data, followed by an authenticate
    /// transaction to chain.
    async fn authenticate(self: Arc<Self>, liveness_data: LivenessData) -> Result<()> {
        info!("Bioauth flow - authentication in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await?;

        let response = self
            .robonode_client
            .as_ref()
            .authenticate(AuthenticateRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await
            .map_err(|err| {
                let data = match err {
                    robonode_client::Error::Call(
                        robonode_client::AuthenticateError::FaceScanRejected
                    ) => Some(ShouldRetry.into()),
                    _ => None,
                };

                RpcError {
                    code: RpcErrorCode::ServerError(ErrorCode::Robonode as _),
                    message: format!("request to the robonode failed: {}", err),
                    data,
                }
            })?;

        info!("Bioauth flow - authentication complete");

        info!(message = "We've obtained an auth ticket", auth_ticket = ?response.auth_ticket);

        let validator_key = self.validator_public_key()?.ok_or(RpcError {
            code: RpcErrorCode::ServerError(ErrorCode::MissingValidatorKey as _),
            message: "Validator key not available".to_string(),
            data: None,
        })?;

        let at = sp_api::BlockId::Hash(self.client.info().best_hash);

        let session_keys = self
            .client.runtime_api()
            .generate_session_keys(&at, None)
            .map_err(|err| RpcError {
                code: RpcErrorCode::ServerError(ErrorCode::RuntimeApi as _),
                message: format!("Error generating session keys: {}", err),
                data: None,
            })?;

        self.client.runtime_api().rotate_session_keys(&at, &validator_key, session_keys)
            .map_err(|err| RpcError {
                code: RpcErrorCode::ServerError(ErrorCode::RuntimeApi as _),
                message: format!("Error rotating session keys: {}", err),
                data: None,
            })?;

        let ext = self
            .client
            .runtime_api()
            .create_authenticate_extrinsic(
                &at,
                response.auth_ticket.into(),
                response.auth_ticket_signature.into(),
            )
            .map_err(|err| RpcError {
                code: RpcErrorCode::ServerError(ErrorCode::RuntimeApi as _),
                message: format!("Error creating auth extrinsic: {}", err),
                data: None,
            })?;

        self.pool
            .submit_and_watch(
                &at,
                sp_runtime::transaction_validity::TransactionSource::Local,
                ext,
            )
            .await
            .map_err(map_txpool_error)?;

        info!("Bioauth flow - authenticate transaction complete");

        Ok(())
    }

    /// Try to extract the validator key.
    fn validator_public_key(&self) -> Result<Option<ValidatorKeyExtractor::PublicKeyType>> {
         self
            .validator_key_extractor
            .extract_validator_key()
            .map_err(|error| {
                tracing::error!(
                    message = "Unable to extract own key at bioauth flow RPC",
                    ?error
                );
                RpcError {
                    code: RpcErrorCode::ServerError(ErrorCode::ValidatorKeyExtraction as _),
                    message: "Unable to extract own key".into(),
                    data: None,
                }
            })

    }

    /// Return the opaque liveness data and corresponding signature.
    async fn sign(&self, liveness_data: &LivenessData) -> Result<(OpaqueLivenessData, Vec<u8>)> {
        let opaque_liveness_data = OpaqueLivenessData::from(liveness_data);
        let validator_key = self.validator_public_key()?.ok_or(RpcError {
            code: RpcErrorCode::ServerError(ErrorCode::MissingValidatorKey as _),
            message: "Validator key not available".to_string(),
            data: None,
        })?;
        let signer = self.validator_signer_factory.new_signer(validator_key);

        let signature = signer
            .sign(&opaque_liveness_data)
            .await
            .map_err(|error| {
                tracing::error!(
                    message = "Signing failed",
                    ?error
                );
                RpcError {
                    code: RpcErrorCode::ServerError(ErrorCode::Signer as _),
                    message: "Signing failed".to_string(),
                    data: None,
                }
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

/// Convert a transaction pool error into a human-readable
fn map_txpool_error<T: sc_transaction_pool_api::error::IntoPoolError>(err: T) -> RpcError {
    let code = RpcErrorCode::ServerError(ErrorCode::Transaction as _);

    let err = match err.into_pool_error() {
        Ok(err) => err,
        Err(err) => {
            // This is not a Transaction Pool API Error, but it may be a kind of wrapper type
            // error (i.e. Transaction Pool Error, without the API bit).
            return RpcError {
                code,
                message: format!("Transaction failed: {}", err),
                data: None,
            };
        }
    };

    use sc_transaction_pool_api::error::Error;
    let (kind, message) = match err {
        // Provide some custom-tweaked error messages for a few select cases:
        Error::InvalidTransaction(InvalidTransaction::BadProof) => (
            TransactionPoolErrorKind::AuthTicketSignatureInvalid,
            "Invalid auth ticket signature",
        ),
        Error::InvalidTransaction(InvalidTransaction::Custom(custom_code))
            if custom_code
                == (pallet_bioauth::CustomInvalidTransactionCodes::UnableToParseAuthTicket
                    as u8) =>
        {
            (
                TransactionPoolErrorKind::UnableToParseAuthTicket,
                "Unable to parse a validly signed auth ticket",
            )
        }
        Error::InvalidTransaction(InvalidTransaction::Stale) => (
            TransactionPoolErrorKind::NonceAlreadyUsed,
            "The auth ticket you provided has already been used",
        ),
        Error::InvalidTransaction(InvalidTransaction::Future) => (
            TransactionPoolErrorKind::AlreadyAuthenticated,
            "Active authentication exists currently, and you can't authenticate again yet",
        ),
        // For the rest cases, fallback to the native error rendering.
        err => {
            return RpcError {
                code,
                message: format!("Transaction failed: {}", err),
                data: None,
            }
        }
    };

    RpcError {
        code,
        message: message.to_string(),
        data: Some(
            TransactionPoolErrorDetails {
                inner_error: err.to_string(),
                kind,
            }
            .into(),
        ),
    }
}
