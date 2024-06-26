//! RPC interface for the author extension logic.

use std::marker::PhantomData;
use std::sync::Arc;

use author_ext_api::AuthorExtApi;
use bioauth_keys::traits::KeyExtractor as KeyExtractorT;
use errors::{
    get_validator_public_key::Error as GetValidatorPublicKeyError, set_keys::Error as SetKeysError,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
};
use rpc_deny_unsafe::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool as TransactionPoolT;
use sp_api::{BlockT, Encode, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use tracing::*;

mod error_data;
mod errors;

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

        let errtype = |val: errors::set_keys::Error<TransactionPool::Error>| val;

        let validator_key =
            rpc_validator_key_logic::validator_public_key(&self.validator_key_extractor)
                .map_err(SetKeysError::KeyExtraction)
                .map_err(errtype)?;

        let at = self.client.info().best_hash;

        let signed_set_keys_extrinsic = self
            .client
            .runtime_api()
            .create_signed_set_keys_extrinsic(at, &validator_key, session_keys.0)
            .map_err(SetKeysError::RuntimeApi)
            .map_err(errtype)?
            .map_err(SetKeysError::ExtrinsicCreation)
            .map_err(errtype)?;

        let _ = self
            .pool
            .submit_and_watch(
                &sp_api::BlockId::Hash(at),
                sp_runtime::transaction_validity::TransactionSource::Local,
                signed_set_keys_extrinsic,
            )
            .await
            .map_err(SetKeysError::AuthorExtTx)
            .map_err(errtype)?;

        info!("Author extension - setting keys transaction complete");

        Ok(())
    }

    async fn get_validator_public_key(&self) -> RpcResult<ValidatorKeyExtractor::PublicKeyType> {
        let validator_public_key =
            rpc_validator_key_logic::validator_public_key(&self.validator_key_extractor)
                .map_err(GetValidatorPublicKeyError::KeyExtraction)?;

        Ok(validator_public_key)
    }
}
