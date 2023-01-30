//! The `authenticate` method error.

use rpc_validator_key_logic::Error as ValidatorKeyError;
use sp_api::ApiError;
use sp_runtime::transaction_validity::InvalidTransaction;

use super::{api_error_code, sign::Error as SignError};
use crate::error_data::{self, BioauthTxErrorDetails};

/// The `authenticate` method error kinds.
#[derive(Debug)]
pub enum Error<TxPoolError: sc_transaction_pool_api::error::IntoPoolError> {
    /// An error that can occur during validator key extraction.
    KeyExtraction(ValidatorKeyError),
    /// An error that can occur during signing process.
    Sign(SignError),
    /// An error that can occur during doing a call into robonode.
    Robonode(robonode_client::Error<robonode_client::AuthenticateError>),
    /// An error that can occur during doing a call into runtime api.
    RuntimeApi(ApiError),
    /// An error that can occur with transaction pool logic.
    BioauthTx(TxPoolError),
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
            Error::Sign(err) => rpc_error_response::simple(api_error_code::SIGN, err.to_string()),
            Error::Robonode(
                err @ robonode_client::Error::Call(
                    robonode_client::AuthenticateError::FaceScanRejected,
                ),
            ) => rpc_error_response::data(
                api_error_code::ROBONODE,
                err.to_string(),
                error_data::ShouldRetry,
            ),
            Error::Robonode(err) => {
                rpc_error_response::simple(api_error_code::ROBONODE, err.to_string())
            }
            Error::RuntimeApi(err) => {
                rpc_error_response::simple(api_error_code::RUNTIME_API, err.to_string())
            }
            Error::BioauthTx(err) => {
                let (message, data) = map_txpool_error(err);
                rpc_error_response::raw(api_error_code::TRANSACTION, message, data)
            }
        }
    }
}

/// Convert a transaction pool error into a human-readable.
fn map_txpool_error<T: sc_transaction_pool_api::error::IntoPoolError>(
    err: T,
) -> (String, Option<error_data::BioauthTxErrorDetails>) {
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
        Error::InvalidTransaction(InvalidTransaction::BadProof) => (
            error_data::BioauthTxErrorKind::AuthTicketSignatureInvalid,
            "Invalid auth ticket signature",
        ),
        Error::InvalidTransaction(InvalidTransaction::Custom(custom_code))
            if custom_code
                == (pallet_bioauth::CustomInvalidTransactionCodes::UnableToParseAuthTicket
                    as u8) =>
        {
            (
                error_data::BioauthTxErrorKind::UnableToParseAuthTicket,
                "Unable to parse a validly signed auth ticket",
            )
        }
        Error::InvalidTransaction(InvalidTransaction::Stale) => (
            error_data::BioauthTxErrorKind::NonceAlreadyUsed,
            "The auth ticket you provided has already been used",
        ),
        Error::InvalidTransaction(InvalidTransaction::Future) => (
            error_data::BioauthTxErrorKind::AlreadyAuthenticated,
            "Active authentication exists currently, and you can't authenticate again yet",
        ),
        // For the rest cases, fallback to simple error rendering.
        err => {
            return (format!("Transaction failed: {}", err), None);
        }
    };

    let data = BioauthTxErrorDetails {
        inner_error: err.to_string(),
        kind,
        message,
    };

    // Rewrite the error message for more human-readable errors while the frontend doesn't support
    // the custom data parsing.
    (message.to_owned(), Some(data))
}
