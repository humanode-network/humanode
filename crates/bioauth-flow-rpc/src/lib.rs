//! The bioauth flow RPC implementation.
//!
//! It is the logic of communication between the humanode (aka humanode-peer),
//! the app on the handheld device that performs the biometric capture,
//! and the robonode server that issues auth tickets.

use std::marker::PhantomData;
use std::sync::Arc;

use bioauth_flow_api::BioauthFlowApi;
use bioauth_keys::traits::KeyExtractor as KeyExtractorT;
use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use robonode_client::{AuthenticateRequest, EnrollRequest};
use rpc_deny_unsafe::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool as TransactionPoolT;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sp_api::{BlockT, Decode, Encode, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::transaction_validity::InvalidTransaction;
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
enum ApiErrorCode {
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
    /// Try to extract the validator key.
    fn validator_public_key(&self) -> RpcResult<Option<ValidatorKeyExtractor::PublicKeyType>> {
        self.validator_key_extractor
            .extract_key()
            .map_err(|error| {
                tracing::error!(
                    message = "Unable to extract own key at bioauth flow RPC",
                    ?error
                );
                errors::ValidatorError::ValidatorKeyExtraction.into()
            })
    }
    /// Return the opaque liveness data and corresponding signature.
    async fn sign(&self, liveness_data: &LivenessData) -> RpcResult<(OpaqueLivenessData, Vec<u8>)> {
        let opaque_liveness_data = OpaqueLivenessData::from(liveness_data);
        let validator_key =
            self.validator_public_key()?
                .ok_or_else(|| errors::ValidatorError::MissingValidatorKey)?;
        let signer = self.validator_signer_factory.new_signer(validator_key);

        let signature = signer.sign(&opaque_liveness_data).await.map_err(|error| {
            tracing::error!(message = "Signing failed", ?error);
            JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                ErrorCode::ServerError(ApiErrorCode::Signer as _).code(),
                "Signing failed".to_owned(),
                None::<()>,
            )))
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
            .map_err(|err| JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                ErrorCode::ServerError(ApiErrorCode::Robonode as _).code(),
                format!("Request to the robonode failed: {}", err),
                None::<()>,
            ))))?;
        Ok(res)
    }

    async fn get_facetec_session_token(&self) -> RpcResult<String> {
        let res = self
            .robonode_client
            .as_ref()
            .get_facetec_session_token()
            .await
            .map_err(|err| JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                ErrorCode::ServerError(ApiErrorCode::Robonode as _).code(),
                format!("Request to the robonode failed: {}", err),
                None::<()>,
            ))))?;
        Ok(res.session_token)
    }

    async fn status(&self) -> RpcResult<BioauthStatus<Timestamp>> {
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
            .map_err(|err| JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
                format!("Unable to get status from the runtime: {}", err),
                None::<()>,
            ))))?;

        Ok(status.into())
    }

    async fn enroll(&self, liveness_data: LivenessData) -> RpcResult<()> {
        self.deny_unsafe.check_if_safe()?;

        info!("Bioauth flow - enrolling in progress");

        let (opaque_liveness_data, signature) = self.sign(&liveness_data).await?;

        let public_key = self.validator_public_key()?.ok_or_else(|| JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::MissingValidatorKey as _).code(),
            "Validator key not available".to_string(),
            None::<()>,
        ))))?;
        self.robonode_client
            .as_ref()
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
                public_key: public_key.as_ref(),
            })
            .await
            .map_err(|err| {
                let data: Option<Value>= match err {
                    robonode_client::Error::Call(
                        robonode_client::EnrollError::FaceScanRejected
                    ) => Some(ShouldRetry.into()),
                    _ => None,
                };

                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::ServerError(ApiErrorCode::Robonode as _).code(),
                    format!("Request to the robonode failed: {}", err),
                    data,
                )))
            })?;

        info!("Bioauth flow - enrolling complete");

        Ok(())
    }

    async fn authenticate(&self, liveness_data: LivenessData) -> RpcResult<()> {
        self.deny_unsafe.check_if_safe()?;

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
                let data: Option<Value>= match err {
                    robonode_client::Error::Call(
                        robonode_client::AuthenticateError::FaceScanRejected
                    ) => Some(ShouldRetry.into()),
                    _ => None,
                };

                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::ServerError(ApiErrorCode::Robonode as _).code(),
                    format!("Request to the robonode failed: {}", err),
                    data,
                )))
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
            .map_err(|err| JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
                    format!("Error creating auth extrinsic: {}", err),
                    None::<()>,
                ))))?;

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
}

/// Convert a transaction pool error into a human-readable
fn map_txpool_error<T: sc_transaction_pool_api::error::IntoPoolError>(err: T) -> JsonRpseeError {
    let code = ErrorCode::ServerError(ApiErrorCode::Transaction as _);

    let err = match err.into_pool_error() {
        Ok(err) => err,
        Err(err) => {
            // This is not a Transaction Pool API Error, but it may be a kind of wrapper type
            // error (i.e. Transaction Pool Error, without the API bit).
            return JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                code.code(),
                format!("Transaction failed: {}", err),
                None::<()>,
            )));
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
            return JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                code.code(),
                format!("Transaction failed: {}", err),
                None::<()>,
            )))
        }
    };

    JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
        code.code(),
        message.to_string(),
        Some(TransactionPoolErrorDetails {
            inner_error: err.to_string(),
            kind,
        }),
    )))
}
