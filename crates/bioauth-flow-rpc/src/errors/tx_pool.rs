//! The transaction pool error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde::Serialize;

use super::ApiErrorCode;

/// The transaction pool error kinds.
#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionPoolError {
    /// Auth ticket signature was not valid.
    AuthTicketSignatureInvalid,
    /// We were unable to parse the auth ticket (although its signature was supposed to be
    /// validated by now).
    UnableToParseAuthTicket,
    /// The nonce was already seen by the system.
    NonceAlreadyUsed,
    /// The active authentication issued by this ticket is still on.
    AlreadyAuthenticated,
    /// The transaction failed with custom error.
    Other(String),
}

impl From<TransactionPoolError> for JsonRpseeError {
    fn from(tx_pool_err: TransactionPoolError) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::Transaction as _).code(),
            "Transaction Pool Error",
            Some(tx_pool_err),
        )))
    }
}
