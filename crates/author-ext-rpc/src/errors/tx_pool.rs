//! The transaction pool error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

/// The transaction pool error kinds.
#[derive(Debug, thiserror::Error)]
pub enum TransactionPoolError {
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
    fn expected_no_funds_error() {
        let error: JsonRpseeError = TransactionPoolError::NoFunds.into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"no funds\"}";
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
