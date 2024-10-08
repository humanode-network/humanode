//! The `authenticate` method error.

use sp_api::ApiError;
use sp_runtime::transaction_validity::InvalidTransaction;

use crate::error;

/// The `authenticate` method error kinds.
#[derive(Debug)]
pub enum Error<TxPoolError> {
    /// An error that can occur during doing a request to robonode.
    RobonodeRequest(error::shared::FlowBaseError<robonode_client::AuthenticateError>),
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
            Error::RobonodeRequest(err) => {
                err.to_jsonrpsee_error::<_, error::data::ShouldRetry>(|call_error| match call_error
                {
                    robonode_client::AuthenticateError::FaceScanRejectedNoBlob
                    | robonode_client::AuthenticateError::FaceScanRejected(_) => {
                        Some(error::data::ShouldRetry)
                    }

                    robonode_client::AuthenticateError::InvalidLivenessData
                    | robonode_client::AuthenticateError::PersonNotFoundNoBlob
                    | robonode_client::AuthenticateError::PersonNotFound(_)
                    | robonode_client::AuthenticateError::SignatureInvalidNoBlob
                    | robonode_client::AuthenticateError::SignatureInvalid(_)
                    | robonode_client::AuthenticateError::LogicInternalNoBlob
                    | robonode_client::AuthenticateError::LogicInternal(_)
                    | robonode_client::AuthenticateError::UnknownCode(_)
                    | robonode_client::AuthenticateError::Unknown(_) => None,
                })
            }
            Error::RuntimeApi(err) => {
                rpc_error_response::simple(error::code::RUNTIME_API, err.to_string())
            }
            Error::BioauthTx(err) => {
                let (message, data) = map_txpool_error(err);
                rpc_error_response::raw(error::code::TRANSACTION, message, data)
            }
        }
    }
}

/// Convert a transaction pool error into a human-readable.
fn map_txpool_error<T>(err: T) -> (String, Option<error::data::BioauthTxErrorDetails>)
where
    T: sc_transaction_pool_api::error::IntoPoolError,
{
    let err = match err.into_pool_error() {
        Ok(err) => err,
        Err(err) => {
            // This is not a Transaction Pool API Error, but it may be a kind of wrapper type
            // error (i.e. Transaction Pool Error, without the API bit).
            return (format!("Transaction failed: {err}"), None);
        }
    };

    use sc_transaction_pool_api::error::Error;
    let (kind, message) = match err {
        // Provide some custom-tweaked error messages for a few select cases:
        Error::InvalidTransaction(InvalidTransaction::BadProof) => (
            error::data::BioauthTxErrorKind::AuthTicketSignatureInvalid,
            "Invalid auth ticket signature",
        ),
        Error::InvalidTransaction(InvalidTransaction::Custom(custom_code))
            if custom_code
                == (pallet_bioauth::CustomInvalidTransactionCodes::UnableToParseAuthTicket
                    as u8) =>
        {
            (
                error::data::BioauthTxErrorKind::UnableToParseAuthTicket,
                "Unable to parse a validly signed auth ticket",
            )
        }
        Error::InvalidTransaction(InvalidTransaction::Stale) => (
            error::data::BioauthTxErrorKind::NonceAlreadyUsed,
            "The auth ticket you provided has already been used",
        ),
        Error::InvalidTransaction(InvalidTransaction::Future) => (
            error::data::BioauthTxErrorKind::AlreadyAuthenticated,
            "Active authentication exists currently, and you can't authenticate again yet",
        ),
        // For the rest cases, fallback to simple error rendering.
        err => {
            return (format!("Transaction failed: {err}"), None);
        }
    };

    let data = error::data::BioauthTxErrorDetails {
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
    use rpc_validator_key_logic::Error as ValidatorKeyError;

    use super::*;

    #[test]
    fn error_key_extraction_validator_key_extraction() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::RobonodeRequest(
                error::shared::FlowBaseError::KeyExtraction(
                    ValidatorKeyError::ValidatorKeyExtraction,
                ),
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
            Error::<sc_transaction_pool_api::error::Error>::RobonodeRequest(
                error::shared::FlowBaseError::KeyExtraction(ValidatorKeyError::MissingValidatorKey),
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
    fn error_sign() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::RobonodeRequest(
                error::shared::FlowBaseError::Sign(error::Sign::SigningFailed),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":100,\"message\":\"signing failed\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_face_scan_rejected() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::RobonodeRequest(
                error::shared::FlowBaseError::RobonodeClient(robonode_client::Error::Call(
                    robonode_client::AuthenticateError::FaceScanRejectedNoBlob,
                )),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: face scan rejected\",\"data\":{\"shouldRetry\":true}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_logic_internal() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::RobonodeRequest(
                error::shared::FlowBaseError::RobonodeClient(robonode_client::Error::Call(
                    robonode_client::AuthenticateError::LogicInternalNoBlob,
                )),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: logic internal error\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_robonode_other() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::RobonodeRequest(
                error::shared::FlowBaseError::RobonodeClient(robonode_client::Error::Call(
                    robonode_client::AuthenticateError::Unknown("test".to_owned()),
                )),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: unknown error: test\"}";
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
    fn error_bioauth_tx_auth_ticket_signature_invalid() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::BioauthTx(
                sc_transaction_pool_api::error::Error::InvalidTransaction(
                    InvalidTransaction::BadProof,
                ),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"Invalid auth ticket signature\",\"data\":{\"kind\":\"AUTH_TICKET_SIGNATURE_INVALID\",\"message\":\"Invalid auth ticket signature\",\"innerError\":\"Invalid transaction validity: InvalidTransaction::BadProof\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_bioauth_tx_unable_to_parse_auth_ticket() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::BioauthTx(
                sc_transaction_pool_api::error::Error::InvalidTransaction(
                    InvalidTransaction::Custom(
                        pallet_bioauth::CustomInvalidTransactionCodes::UnableToParseAuthTicket
                            as u8,
                    ),
                ),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"Unable to parse a validly signed auth ticket\",\"data\":{\"kind\":\"UNABLE_TO_PARSE_AUTH_TICKET\",\"message\":\"Unable to parse a validly signed auth ticket\",\"innerError\":\"Invalid transaction validity: InvalidTransaction::Custom(116)\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_bioauth_tx_nonce_already_used() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::BioauthTx(
                sc_transaction_pool_api::error::Error::InvalidTransaction(
                    InvalidTransaction::Stale,
                ),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"The auth ticket you provided has already been used\",\"data\":{\"kind\":\"NONCE_ALREADY_USED\",\"message\":\"The auth ticket you provided has already been used\",\"innerError\":\"Invalid transaction validity: InvalidTransaction::Stale\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_bioauth_tx_already_authenticated() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::BioauthTx(
                sc_transaction_pool_api::error::Error::InvalidTransaction(
                    InvalidTransaction::Future,
                ),
            )
            .into();
        let error: ErrorObject = error.into();

        let expected_error_message = "{\"code\":400,\"message\":\"Active authentication exists currently, and you can't authenticate again yet\",\"data\":{\"kind\":\"ALREADY_AUTHENTICATED\",\"message\":\"Active authentication exists currently, and you can't authenticate again yet\",\"innerError\":\"Invalid transaction validity: InvalidTransaction::Future\"}}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }

    #[test]
    fn error_bioauth_tx_other() {
        let error: jsonrpsee::core::Error =
            Error::<sc_transaction_pool_api::error::Error>::BioauthTx(
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
