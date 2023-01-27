//! The transaction pool error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use sp_runtime::transaction_validity::InvalidTransaction;

use super::ApiErrorCode;

/// The transaction pool error kinds.
#[derive(Debug, thiserror::Error)]
pub enum BioauthTxError {
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
    /// The native transaction pool error.
    #[error("transaction pool error: {0}")]
    Native(String),
    /// The unexpected transaction pool error.
    #[error("unexpected transaction pool error: {0}")]
    Unexpected(String),
}

impl From<BioauthTxError> for JsonRpseeError {
    fn from(err: BioauthTxError) -> Self {
        JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
            ErrorCode::ServerError(ApiErrorCode::Transaction as _).code(),
            err.to_string(),
            None::<()>,
        )))
    }
}

impl<T: sc_transaction_pool_api::error::IntoPoolError> From<T> for BioauthTxError {
    fn from(err: T) -> Self {
        let err = match err.into_pool_error() {
            Ok(err) => err,
            Err(err) => {
                // This is not a Transaction Pool API Error, but it may be a kind of wrapper type
                // error (i.e. Transaction Pool Error, without the API bit).
                return BioauthTxError::Unexpected(err.to_string());
            }
        };

        use sc_transaction_pool_api::error::Error;
        match err {
            // Provide some custom-tweaked error messages for a few select cases:
            Error::InvalidTransaction(InvalidTransaction::BadProof) => {
                BioauthTxError::AuthTicketSignatureInvalid
            }
            Error::InvalidTransaction(InvalidTransaction::Custom(custom_code))
                if custom_code
                    == (pallet_bioauth::CustomInvalidTransactionCodes::UnableToParseAuthTicket
                        as u8) =>
            {
                BioauthTxError::UnableToParseAuthTicket
            }
            Error::InvalidTransaction(InvalidTransaction::Stale) => {
                BioauthTxError::NonceAlreadyUsed
            }
            Error::InvalidTransaction(InvalidTransaction::Future) => {
                BioauthTxError::AlreadyAuthenticated
            }
            // For the rest cases, fallback to the native error rendering.
            err => BioauthTxError::Native(err.to_string()),
        }
    }
}
