//! RPC interface for the author extension logic.

use std::marker::PhantomData;
use std::sync::Arc;

use author_ext_api::AuthorExtApi;
use bioauth_consensus::ValidatorKeyExtractor as ValidatorKeyExtractorT;
use futures::FutureExt;
use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode as RpcErrorCode;
use jsonrpc_derive::rpc;
use sc_transaction_pool_api::{error::IntoPoolError, TransactionPool as TransactionPoolT};
use sp_api::{BlockT, Encode, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use tracing::*;

/// A result type that wraps.
pub type Result<T> = std::result::Result<T, RpcError>;

/// A futures that resolves to the specified `T`, or an [`RpcError`].
pub type FutureResult<T> = jsonrpc_core::BoxFuture<Result<T>>;

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
enum ErrorCode {
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
#[rpc]
pub trait AuthorExtRpcApi {
    /// Set_keys with provided session keys data.
    #[rpc(name = "authorExt_setKeys")]
    fn set_keys(&self, session_keys: Bytes) -> FutureResult<()>;
}

/// The RPC implementation.
pub struct AuthorExt<ValidatorKeyExtractor, Client, Block, TransactionPool> {
    /// The underlying implementation.
    /// We have to wrap it with `Arc` because `jsonrpc-core` doesn't allow us to use `self` for
    /// the duration of the future; we `clone` the `Arc` to get the `'static` lifetime to
    /// a shared `Inner` instead.
    inner: Arc<Inner<ValidatorKeyExtractor, Client, Block, TransactionPool>>,
}

impl<ValidatorKeyExtractor, Client, Block, TransactionPool>
    AuthorExt<ValidatorKeyExtractor, Client, Block, TransactionPool>
{
    /// Create a new [`AuthorExt`] API implementation.
    pub fn new(
        validator_key_extractor: ValidatorKeyExtractor,
        client: Arc<Client>,
        pool: Arc<TransactionPool>,
    ) -> Self {
        let inner = Inner {
            validator_key_extractor,
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
        F: FnOnce(Arc<Inner<ValidatorKeyExtractor, Client, Block, TransactionPool>>) -> Fut,
        Fut: std::future::Future<Output = Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        let inner = Arc::clone(&self.inner);
        f(inner).boxed()
    }
}

impl<ValidatorKeyExtractor, Client, Block, TransactionPool> AuthorExtRpcApi
    for AuthorExt<ValidatorKeyExtractor, Client, Block, TransactionPool>
where
    Client: Send + Sync + 'static,
    Block: Send + Sync + 'static,
    TransactionPool: Send + Sync + 'static,
    ValidatorKeyExtractor: Send + Sync + 'static,
    ValidatorKeyExtractor::PublicKeyType: Send + Sync + 'static,

    ValidatorKeyExtractor: ValidatorKeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode + AsRef<[u8]>,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client: Send + Sync + 'static,
    Client::Api: AuthorExtApi<Block, ValidatorKeyExtractor::PublicKeyType>,
    Block: BlockT,
    TransactionPool: TransactionPoolT<Block = Block>,
{
    /// See [`Inner::set_keys`].
    fn set_keys(&self, session_keys: Bytes) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.set_keys(session_keys))
    }
}

/// The underlying implementation of the RPC part, extracted into a subobject to work around
/// the common pitfall with the poor async engines implementations of requiring future objects to
/// be static.
/// Stop it people, why do you even use Rust if you do things like this? Ffs...
/// See https://github.com/paritytech/jsonrpc/issues/580
struct Inner<ValidatorKeyExtractor, Client, Block, TransactionPool> {
    /// Provider of the local validator key.
    validator_key_extractor: ValidatorKeyExtractor,
    /// The substrate client, provides access to the runtime APIs.
    client: Arc<Client>,
    /// The transaction pool to use.
    pool: Arc<TransactionPool>,
    /// The phantom types.
    phantom_types: PhantomData<Block>,
}

impl<ValidatorKeyExtractor, Client, Block, TransactionPool>
    Inner<ValidatorKeyExtractor, Client, Block, TransactionPool>
where
    ValidatorKeyExtractor: ValidatorKeyExtractorT,
    ValidatorKeyExtractor::PublicKeyType: Encode + AsRef<[u8]>,
    ValidatorKeyExtractor::Error: std::fmt::Debug,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client: Send + Sync + 'static,
    Client::Api: AuthorExtApi<Block, ValidatorKeyExtractor::PublicKeyType>,
    Block: BlockT,
    TransactionPool: TransactionPoolT<Block = Block>,
{
    /// Submit set_keys transaction to chain with provided session keys data.
    async fn set_keys(self: Arc<Self>, session_keys: Bytes) -> Result<()> {
        info!("Author extension - setting keys in progress");

        let validator_key = self.validator_public_key()?.ok_or(RpcError {
            code: RpcErrorCode::ServerError(ErrorCode::MissingValidatorKey as _),
            message: "Validator key not available".to_string(),
            data: None,
        })?;

        let at = sp_api::BlockId::Hash(self.client.info().best_hash);

        let signed_set_keys_extrinsic = self
            .client
            .runtime_api()
            .create_signed_set_keys_extrinsic(&at, &validator_key, session_keys.0)
            .map_err(|err| RpcError {
                code: RpcErrorCode::ServerError(ErrorCode::RuntimeApi as _),
                message: format!("Runtime error: {}", err),
                data: None,
            })?
            .map_err(|err| match err {
                author_ext_api::CreateSignedSetKeysExtrinsicError::SessionKeysDecodingError => {
                    RpcError {
                        code: RpcErrorCode::ServerError(ErrorCode::RuntimeApi as _),
                        message: "Error during session keys decoding".to_owned(),
                        data: None,
                    }
                }
                author_ext_api::CreateSignedSetKeysExtrinsicError::SignedExtrinsicCreationError => {
                    RpcError {
                        code: RpcErrorCode::ServerError(ErrorCode::RuntimeApi as _),
                        message: "Error during the creation of the signed set keys extrinsic".to_owned(),
                        data: None,
                    }
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
                RpcError {
                    code: RpcErrorCode::ServerError(ErrorCode::Transaction as _),
                    message,
                    data: None,
                }
            })?;

        info!("Author extension - setting keys transaction complete");

        Ok(())
    }

    /// Try to extract the validator key.
    fn validator_public_key(&self) -> Result<Option<ValidatorKeyExtractor::PublicKeyType>> {
        self.validator_key_extractor
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
}
