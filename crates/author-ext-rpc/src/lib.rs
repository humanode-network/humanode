//! RPC interface for the author extension logic.

use std::marker::PhantomData;
use std::sync::Arc;

use author_ext_api::AuthorExtApi;
use bioauth_keys::traits::KeyExtractor as KeyExtractorT;
use jsonrpsee::{
    core::{async_trait, Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use rpc_deny_unsafe::DenyUnsafe;
use sc_transaction_pool_api::{error::IntoPoolError, TransactionPool as TransactionPoolT};
use sp_api::{BlockT, Encode, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use tracing::*;

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
enum ApiErrorCode {
    /// Call to runtime api has failed.
    RuntimeApi = 300,
    /// Set_keys transaction has failed.
    Transaction = 400,
    /// Validator key is not available.
    MissingValidatorKey = 500,
    /// Validator key extraction has failed.
    ValidatorKeyExtraction = 600,
}

/// The API exposed via JSON-RPC.
#[rpc(server)]
pub trait AuthorExt<VPK> {
    /// Set_keys with provided session keys data.
    #[method(name = "authorExt_setKeys")]
    async fn set_keys(&self, session_keys: Bytes) -> RpcResult<()>;

    /// Provide validator public key.
    #[method(name = "authorExt_getValidatorPublicKey")]
    async fn get_validator_public_key(&self) -> RpcResult<VPK>;
}

/// The RPC implementation.
pub struct AuthorExt<ValidatorKeyExtractor, Client, Block, TransactionPool> {
    /// Provider of the local validator key.
    validator_key_extractor: ValidatorKeyExtractor,
    /// The substrate client, provides access to the runtime APIs.
    client: Arc<Client>,
    /// The transaction pool to use.
    pool: Arc<TransactionPool>,
    /// Whether to deny unsafe calls or not.
    deny_unsafe: DenyUnsafe,
    /// The phantom types.
    phantom_types: PhantomData<Block>,
}

impl<ValidatorKeyExtractor, Client, Block, TransactionPool>
    AuthorExt<ValidatorKeyExtractor, Client, Block, TransactionPool>
{
    /// Create a new [`AuthorExt`] API implementation.
    pub fn new(
        validator_key_extractor: ValidatorKeyExtractor,
        client: Arc<Client>,
        pool: Arc<TransactionPool>,
        deny_unsafe: DenyUnsafe,
    ) -> Self {
        Self {
            validator_key_extractor,
            client,
            pool,
            deny_unsafe,
            phantom_types: PhantomData,
        }
    }
}

impl<ValidatorKeyExtractor, Client, Block, TransactionPool>
    AuthorExt<ValidatorKeyExtractor, Client, Block, TransactionPool>
where
    ValidatorKeyExtractor: KeyExtractorT,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
{
    /// Try to extract the validator key.
    fn validator_public_key(&self) -> RpcResult<Option<ValidatorKeyExtractor::PublicKeyType>> {
        self.validator_key_extractor.extract_key().map_err(|error| {
            tracing::error!(
                message = "Unable to extract own key at author extension RPC",
                ?error
            );
            JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                ErrorCode::ServerError(ApiErrorCode::ValidatorKeyExtraction as _).code(),
                "Unable to extract own key".to_owned(),
                None::<()>,
            )))
        })
    }
}

#[async_trait]
impl<ValidatorKeyExtractor, Client, Block, TransactionPool>
    AuthorExtServer<ValidatorKeyExtractor::PublicKeyType>
    for AuthorExt<ValidatorKeyExtractor, Client, Block, TransactionPool>
where
    Client: Send + Sync + 'static,
    Block: Send + Sync + 'static,
    TransactionPool: Send + Sync + 'static,
    ValidatorKeyExtractor: Send + Sync + 'static,
    ValidatorKeyExtractor::PublicKeyType: Send + Sync + 'static,

    ValidatorKeyExtractor: KeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode + AsRef<[u8]>,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client: Send + Sync + 'static,
    Client::Api: AuthorExtApi<Block, ValidatorKeyExtractor::PublicKeyType>,
    Block: BlockT,
    TransactionPool: TransactionPoolT<Block = Block>,
{
    async fn set_keys(&self, session_keys: Bytes) -> RpcResult<()> {
        self.deny_unsafe.check_if_safe()?;

        info!("Author extension - setting keys in progress");

        let validator_key = self.validator_public_key()?.ok_or_else(|| {
            JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                ErrorCode::ServerError(ApiErrorCode::MissingValidatorKey as _).code(),
                "Validator key not available".to_owned(),
                None::<()>,
            )))
        })?;

        let at = sp_api::BlockId::Hash(self.client.info().best_hash);

        let signed_set_keys_extrinsic = self
            .client
            .runtime_api()
            .create_signed_set_keys_extrinsic(&at, &validator_key, session_keys.0)
            .map_err(|err| {
                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
                    format!("Runtime error: {}", err),
                    None::<()>,
                )))
            })?
            .map_err(|err| match err {
                author_ext_api::CreateSignedSetKeysExtrinsicError::SessionKeysDecoding(err) => {
                    JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                        ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
                        format!("Error during session keys decoding: {}", err),
                        None::<()>,
                    )))
                }
                author_ext_api::CreateSignedSetKeysExtrinsicError::SignedExtrinsicCreation => {
                    JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                        ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
                        "Error during the creation of the signed set keys extrinsic".to_owned(),
                        None::<()>,
                    )))
                }
            })?;

        self.pool
            .submit_and_watch(
                &at,
                sp_runtime::transaction_validity::TransactionSource::Local,
                signed_set_keys_extrinsic,
            )
            .await
            .map_err(|e| {
                let message = e.into_pool_error().map_or_else(
                    |err| format!("Transaction pool error: {}", err),
                    |err| format!("Unexpected transaction pool error: {}", err),
                );
                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::ServerError(ApiErrorCode::Transaction as _).code(),
                    message,
                    None::<()>,
                )))
            })?;

        info!("Author extension - setting keys transaction complete");

        Ok(())
    }

    async fn get_validator_public_key(&self) -> RpcResult<ValidatorKeyExtractor::PublicKeyType> {
        let validator_public_key = self.validator_public_key()?.ok_or_else(|| {
            JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                ErrorCode::ServerError(ApiErrorCode::MissingValidatorKey as _).code(),
                "Validator key not available".to_owned(),
                None::<()>,
            )))
        })?;

        Ok(validator_public_key)
    }
}
