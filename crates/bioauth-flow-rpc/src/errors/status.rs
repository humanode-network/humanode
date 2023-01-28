//! The `status` method error.

use rpc_validator_key_logic::ValidatorKeyError;
use sp_api::ApiError;

use super::{app, ApiErrorCode};

/// The `status` method error kinds.
#[derive(Debug)]
pub enum StatusError {
    /// An error that can occur during validator key extraction.
    /// Specifically the validator key extraction failure, not the missing key.
    ValidatorKeyExtraction,
    /// An error that can occur during doing a call into runtime api.
    RuntimeApi(ApiError),
}

impl From<StatusError> for jsonrpsee::core::Error {
    fn from(err: StatusError) -> Self {
        match err {
            StatusError::ValidatorKeyExtraction => app::simple(
                ApiErrorCode::ValidatorKeyExtraction,
                ValidatorKeyError::ValidatorKeyExtraction.to_string(),
            ),
            StatusError::RuntimeApi(err) => app::simple(
                ApiErrorCode::RuntimeApi,
                format!("unable to get status from the runtime: {err}"),
            ),
        }
    }
}
