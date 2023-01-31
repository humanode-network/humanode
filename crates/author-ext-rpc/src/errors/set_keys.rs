//! The `set_keys` method error.

use author_ext_api::CreateSignedSetKeysExtrinsicError;
use rpc_validator_key_logic::Error as ValidatorKeyError;
use sp_api::ApiError;
use sp_runtime::transaction_validity::InvalidTransaction;

use super::api_error_code;
use crate::error_data::{self, AuthorExtTxErrorDetails};

/// The `set_keys` method error kinds.
#[derive(Debug)]
pub enum Error<TxPoolError: sc_transaction_pool_api::error::IntoPoolError> {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during doing a call into runtime api.
    RuntimeApi(ApiError),
    /// An error that can occur during signed `set_keys` extrinsic creation.
    ExtrinsicCreation(CreateSignedSetKeysExtrinsicError),
    /// An error that can occur with transaction pool logic.
    AuthorExtTx(TxPoolError),
}

impl<TxPoolError> From<Error<TxPoolError>> for jsonrpsee::core::Error
where
    TxPoolError: sc_transaction_pool_api::error::IntoPoolError,
{
    fn from(err: Error<TxPoolError>) -> Self {
        match err {
            Error::KeyExtraction(err @ ValidatorKeyError::MissingValidatorKey) => {
                rpc_error_response::data(
                    api_error_code::MISSING_VALIDATOR_KEY,
                    err.to_string(),
                    rpc_validator_key_logic::error_data::ValidatorKeyNotAvailable,
                )
            }
            Error::KeyExtraction(err @ ValidatorKeyError::ValidatorKeyExtraction) => {
                rpc_error_response::simple(
                    api_error_code::VALIDATOR_KEY_EXTRACTION,
                    err.to_string(),
                )
            }
            Error::RuntimeApi(err) => {
                rpc_error_response::simple(api_error_code::RUNTIME_API, err.to_string())
            }
            Error::ExtrinsicCreation(
                ref _err @ CreateSignedSetKeysExtrinsicError::SessionKeysDecoding(ref err_details),
            ) => rpc_error_response::simple(
                api_error_code::RUNTIME_API,
                format!("Error during session keys decoding: {err_details}"),
            ),
            Error::ExtrinsicCreation(
                _err @ CreateSignedSetKeysExtrinsicError::SignedExtrinsicCreation,
            ) => rpc_error_response::simple(
                api_error_code::RUNTIME_API,
                "Error during the creation of the signed set keys extrinsic".to_owned(),
            ),
            Error::AuthorExtTx(err) => {
                let (message, data) = map_txpool_error(err);
                rpc_error_response::raw(api_error_code::TRANSACTION, message, data)
            }
        }
    }
}

/// Convert a transaction pool error into a human-readable.
fn map_txpool_error<T: sc_transaction_pool_api::error::IntoPoolError>(
    err: T,
) -> (String, Option<error_data::AuthorExtTxErrorDetails>) {
    let err = match err.into_pool_error() {
        Ok(err) => err,
        Err(err) => {
            // This is not a Transaction Pool API Error, but it may be a kind of wrapper type
            // error (i.e. Transaction Pool Error, without the API bit).
            return (format!("Transaction failed: {}", err), None);
        }
    };

    use sc_transaction_pool_api::error::Error;
    let (kind, message) = match err {
        // Provide some custom-tweaked error messages for a few select cases:
        Error::InvalidTransaction(InvalidTransaction::Payment) => {
            (error_data::AuthorExtTxErrorKind::NoFunds, "No funds")
        }
        // For the rest cases, fallback to the native error rendering.
        err => {
            return (format!("Transaction failed: {}", err), None);
        }
    };

    let data = AuthorExtTxErrorDetails {
        inner_error: err.to_string(),
        kind,
        message,
    };

    // Rewrite the error message for more human-readable errors while the frontend doesn't support
    // the custom data parsing.
    (message.to_owned(), Some(data))
}

#[cfg(test)]
mod tests {

    use jsonrpsee::types::ErrorObject;

    use super::*;

    #[test]
    fn error_key_extraction_validator_key_extraction() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::KeyExtraction(
                ValidatorKeyError::ValidatorKeyExtraction,
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":600,\"message\":\"unable to extract own key\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_key_extraction_missing_validator_key() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::KeyExtraction(
                ValidatorKeyError::MissingValidatorKey,
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":500,\"message\":\"validator key not available\",\"data\":{\"validatorKeyNotAvailable\":true}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_runtime_api() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::RuntimeApi(ApiError::Application(
                "test".into(),
            ))
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":300,\"message\":\"test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_extrinsic_creation_session_keys_decoding() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::ExtrinsicCreation(
                CreateSignedSetKeysExtrinsicError::SessionKeysDecoding("test".to_owned()),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":300,\"message\":\"Error during session keys decoding: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_extrinsic_creation_signed_extrinsic_creation() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::ExtrinsicCreation(
                CreateSignedSetKeysExtrinsicError::SignedExtrinsicCreation,
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":300,\"message\":\"Error during the creation of the signed set keys extrinsic\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_author_ext_tx_no_funds() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::AuthorExtTx(
                sc_transaction_pool_api::error::Error::InvalidTransaction(
                    InvalidTransaction::Payment,
                ),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"No funds\",\"data\":{\"kind\":\"NO_FUNDS\",\"message\":\"No funds\",\"innerError\":\"Invalid transaction validity: InvalidTransaction::Payment\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_author_ext_tx_other() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::AuthorExtTx(
                sc_transaction_pool_api::error::Error::RejectedFutureTransaction,
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"Transaction failed: The pool is not accepting future transactions\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
