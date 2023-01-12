//! The transaction pool error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

/// The transaction pool error kinds.
#[derive(Debug, thiserror::Error)]
pub enum TransactionPoolError {
    /// Auth ticket signature was not valid.
    #[error("invalid auth ticket signature")]
    AuthTicketSignatureInvalid,
    /// We were unable to parse the auth ticket (although its signature was supposed to be
    /// validated by now).
    #[error("unable to parse auth ticket")]
    UnableToParseAuthTicket,
    /// The nonce was already seen by the system.
    #[error("nonce already used")]
    NonceAlreadyUsed,
    /// The active authentication issued by this ticket is still on.
    #[error("already authenticated")]
    AlreadyAuthenticated,
    /// The inability to pay some fees (e.g. account balance too low).
    #[error("no funds")]
    NoFunds,
    /// The native transaction pool error.
    #[error("native transaction pool error: {0}")]
    Native(String),
    /// The unexpected transaction pool error.
    #[error("unexpected transaction pool error: {0}")]
    Unexpected(String),
}

impl From<TransactionPoolError> for JsonRpseeError {
    fn from(err: TransactionPoolError) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::Transaction as _).code(),
            err.to_string(),
            None::<()>,
        )))
    }
}
