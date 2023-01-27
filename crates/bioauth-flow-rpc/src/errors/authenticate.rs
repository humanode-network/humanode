//! The `authenticate` method error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use sp_api::ApiError;

use super::{robonode::RobonodeError, signer::SignerError, tx_pool::BioauthTxError, ApiErrorCode};

/// The `authenticate` method error kinds.
#[derive(Debug)]
pub enum AuthenticateError {
    /// An error that can occur during doing a call into robonode.
    Robonode(RobonodeError),
    /// An error that can occur during doing a call into runtime api.
    RuntimeApi(ApiError),
    /// An error that can occur during signing process.
    Signer(SignerError),
    /// An error that can occur with transaction pool logic.
    TxPool(BioauthTxError),
}

impl From<AuthenticateError> for JsonRpseeError {
    fn from(err: AuthenticateError) -> Self {
        match err {
            AuthenticateError::Robonode(err) => err.into(),
            AuthenticateError::RuntimeApi(err) => {
                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::ServerError(ApiErrorCode::RuntimeApi as _).code(),
                    err.to_string(),
                    None::<()>,
                )))
            }
            AuthenticateError::Signer(err) => err.into(),
            AuthenticateError::TxPool(err) => err.into(),
        }
    }
}
