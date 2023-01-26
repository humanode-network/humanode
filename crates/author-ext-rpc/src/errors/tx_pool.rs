//! The transaction pool error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use sp_runtime::transaction_validity::InvalidTransaction;

use super::ApiErrorCode;

/// The transaction pool error kinds.
#[derive(Debug, thiserror::Error)]
pub enum AuthorExtTxError {
    /// The inability to pay some fees (e.g. account balance too low).
    #[error("no funds")]
    NoFunds,
    /// The native transaction pool error.
    #[error("transaction pool error: {0}")]
    Native(String),
    /// The unexpected transaction pool error.
    #[error("unexpected transaction pool error: {0}")]
    Unexpected(String),
}

impl From<AuthorExtTxError> for JsonRpseeError {
    fn from(err: AuthorExtTxError) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::Transaction as _).code(),
            err.to_string(),
            None::<()>,
        )))
    }
}

impl<T: sc_transaction_pool_api::error::IntoPoolError> From<T> for AuthorExtTxError {
    fn from(err: T) -> Self {
        let err = match err.into_pool_error() {
            Ok(err) => err,
            Err(err) => {
                // This is not a Transaction Pool API Error, but it may be a kind of wrapper type
                // error (i.e. Transaction Pool Error, without the API bit).
                return AuthorExtTxError::Unexpected(err.to_string());
            }
        };

        use sc_transaction_pool_api::error::Error;
        match err {
            // Provide some custom-tweaked error messages for a few select cases:
            Error::InvalidTransaction(InvalidTransaction::Payment) => AuthorExtTxError::NoFunds,
            // For the rest cases, fallback to the native error rendering.
            err => AuthorExtTxError::Native(err.to_string()),
        }
    }
}
