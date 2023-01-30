//! The `set_keys` method error kinds.

use author_ext_api::CreateSignedSetKeysExtrinsicError;
use rpc_validator_key_logic::Error as ValidatorKeyError;
use sp_api::ApiError;
use sp_runtime::transaction_validity::InvalidTransaction;

use super::{app, ApiErrorCode};
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
            Error::KeyExtraction(err @ ValidatorKeyError::MissingValidatorKey) => app::data(
                ApiErrorCode::MissingValidatorKey,
                err.to_string(),
                rpc_validator_key_logic::error_data::ValidatorKeyNotAvailable,
            ),
            Error::KeyExtraction(err @ ValidatorKeyError::ValidatorKeyExtraction) => {
                app::simple(ApiErrorCode::ValidatorKeyExtraction, err.to_string())
            }
            Error::RuntimeApi(err) => app::simple(ApiErrorCode::RuntimeApi, err.to_string()),
            Error::ExtrinsicCreation(
                ref _err @ CreateSignedSetKeysExtrinsicError::SessionKeysDecoding(ref err_details),
            ) => app::simple(
                ApiErrorCode::RuntimeApi,
                format!("Error during session keys decoding: {err_details}"),
            ),
            Error::ExtrinsicCreation(
                _err @ CreateSignedSetKeysExtrinsicError::SignedExtrinsicCreation,
            ) => app::simple(
                ApiErrorCode::RuntimeApi,
                "Error during the creation of the signed set keys extrinsic".to_owned(),
            ),
            Error::AuthorExtTx(err) => {
                let (message, data) = map_txpool_error(err);
                app::raw(ApiErrorCode::Transaction, message, data)
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
