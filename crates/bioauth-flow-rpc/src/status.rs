use super::*;
use crate::errors::api_error_code;

/// The API exposed via JSON-RPC.
#[rpc(server)]
pub trait BioauthStatus<Timestamp> {
    /// Get the current bioauth status.
    #[method(name = "bioauth_status")]
    async fn status(&self) -> RpcResult<BioauthStatus<Timestamp>>;
}

pub struct Server<ValidatorKeyExtractor, Block, Client> {
    /// Provider of the local validator key.
    pub validator_key_extractor: ValidatorKeyExtractor,

    /// The substrate client, provides access to the runtime APIs.
    pub client: Arc<Client>,

    /// The phantom types.
    pub phantom_types: PhantomData<Block>,
}

#[jsonrpsee::core::async_trait]
impl<Timestamp, ValidatorKeyExtractor, Block, Client> BioauthStatusServer<Timestamp>
    for Server<ValidatorKeyExtractor, Block, Client>
where
    ValidatorKeyExtractor: Send + Sync + 'static,
    ValidatorKeyExtractor: bioauth_keys::traits::KeyExtractor,

    Block: BlockT,

    Client: 'static,
    Client: HeaderBackend<Block>,
    Client: ProvideRuntimeApi<Block>,
    Client::Api:
        bioauth_flow_api::BioauthFlowApi<Block, ValidatorKeyExtractor::PublicKeyType, Timestamp>,

    Timestamp: Decode,

    ValidatorKeyExtractor::PublicKeyType: Encode,
{
    async fn status(&self) -> RpcResult<BioauthStatus<Timestamp>> {
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

        let at = self.client.info().best_hash;

        let status = self
            .client
            .runtime_api()
            .bioauth_status(at, &own_key)
            .map_err(StatusError::RuntimeApi)?;

        Ok(status.into())
    }
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

/// The `status` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during validator key extraction.
    /// Specifically the validator key extraction failure, not the missing key.
    ValidatorKeyExtraction,
    /// An error that can occur during doing a call into runtime api.
    RuntimeApi(sp_api::ApiError),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::ValidatorKeyExtraction => rpc_error_response::simple(
                api_error_code::VALIDATOR_KEY_EXTRACTION,
                rpc_validator_key_logic::Error::ValidatorKeyExtraction.to_string(),
            ),
            Error::RuntimeApi(err) => rpc_error_response::simple(
                api_error_code::RUNTIME_API,
                format!("unable to get status from the runtime: {err}"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {

    use jsonrpsee::types::ErrorObject;

    use super::*;

    #[test]
    fn error_validator_key_extraction() {
        let error: jsonrpsee::core::Error = Error::ValidatorKeyExtraction.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":600,\"message\":\"unable to extract own key\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_runtime_api() {
        let error: jsonrpsee::core::Error =
            Error::RuntimeApi(sp_api::ApiError::Application("test".into())).into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":300,\"message\":\"unable to get status from the runtime: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
