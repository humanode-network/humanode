//! RPC interface for the author extension logic.

use std::marker::PhantomData;
use std::sync::Arc;

use author_ext_api::AuthorExtApi;
use bioauth_keys::traits::KeyExtractor as KeyExtractorT;
use humanode_rpc_errors::{RuntimeApiError, TransactionPoolError, ValidatorKeyError};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
};
use rpc_deny_unsafe::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool as TransactionPoolT;
use sp_api::{BlockT, Encode, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::transaction_validity::InvalidTransaction;
use tracing::*;

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
            ValidatorKeyError::ValidatorKeyExtraction.into()
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

        let validator_key = self
            .validator_public_key()?
            .ok_or(ValidatorKeyError::MissingValidatorKey)?;

        let at = sp_api::BlockId::Hash(self.client.info().best_hash);

        let signed_set_keys_extrinsic = self
            .client
            .runtime_api()
            .create_signed_set_keys_extrinsic(&at, &validator_key, session_keys.0)
            .map_err(|err| RuntimeApiError::Runtime(err.to_string()))?
            .map_err(|err| match err {
                author_ext_api::CreateSignedSetKeysExtrinsicError::SessionKeysDecoding(err) => {
                    RuntimeApiError::SessionKeysDecoding(err)
                }
                author_ext_api::CreateSignedSetKeysExtrinsicError::SignedExtrinsicCreation => {
                    RuntimeApiError::CreatingSignedSetKeys
                }
            })?;

        self.pool
            .submit_and_watch(
                &at,
                sp_runtime::transaction_validity::TransactionSource::Local,
                signed_set_keys_extrinsic,
            )
            .await
            .map_err(map_txpool_error)?;

        info!("Author extension - setting keys transaction complete");

        Ok(())
    }

    async fn get_validator_public_key(&self) -> RpcResult<ValidatorKeyExtractor::PublicKeyType> {
        let validator_public_key = self
            .validator_public_key()?
            .ok_or(ValidatorKeyError::MissingValidatorKey)?;

        Ok(validator_public_key)
    }
}

/// Convert a transaction pool error into an author ext related error.
fn map_txpool_error<T: sc_transaction_pool_api::error::IntoPoolError>(
    err: T,
) -> TransactionPoolError {
    let err = match err.into_pool_error() {
        Ok(err) => err,
        Err(err) => {
            // This is not a Transaction Pool API Error, but it may be a kind of wrapper type
            // error (i.e. Transaction Pool Error, without the API bit).
            return TransactionPoolError::Other(err.to_string());
        }
    };

    use sc_transaction_pool_api::error::Error;
    match err {
        // Provide some custom-tweaked error messages for a few select cases:
        Error::InvalidTransaction(InvalidTransaction::Payment) => TransactionPoolError::NoFunds,
        // For the rest cases, fallback to the native error rendering.
        err => TransactionPoolError::Other(err.to_string()),
    }
}
