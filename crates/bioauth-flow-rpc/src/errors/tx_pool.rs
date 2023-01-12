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
    /// The native transaction pool error.
    #[error("transaction pool error: {0}")]
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn expected_auth_ticket_signature_invalid_error() {
        let error: JsonRpseeError = TransactionPoolError::AuthTicketSignatureInvalid.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"invalid auth ticket signature\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_unable_to_parse_auth_ticket_error() {
        let error: JsonRpseeError = TransactionPoolError::UnableToParseAuthTicket.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"unable to parse auth ticket\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_nonce_already_used_error() {
        let error: JsonRpseeError = TransactionPoolError::NonceAlreadyUsed.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"nonce already used\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_already_authenticated_error() {
        let error: JsonRpseeError = TransactionPoolError::AlreadyAuthenticated.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"already authenticated\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_native_error() {
        let error: JsonRpseeError = TransactionPoolError::Native("test".to_string()).into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"transaction pool error: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn expected_unexpected_error() {
        let error: JsonRpseeError = TransactionPoolError::Unexpected("test".to_string()).into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":400,\"message\":\"unexpected transaction pool error: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
