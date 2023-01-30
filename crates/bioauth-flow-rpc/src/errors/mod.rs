//! All bioauth flow error kinds that we expose in the RPC.

pub mod authenticate;
pub mod enroll;
pub mod get_facetec_device_sdk_params;
pub mod get_facetec_session_token;
pub mod sign;
pub mod status;

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
pub enum ApiErrorCode {
    /// Signer has failed.
    Sign = 100,
    /// Request to robonode has failed.
    Robonode = 200,
    /// Call to runtime api has failed.
    RuntimeApi = 300,
    /// Authenticate transaction has failed.
    Transaction = 400,
    /// Validator key is not available.
    MissingValidatorKey = rpc_validator_key_logic::ApiErrorCode::MissingValidatorKey as _,
    /// Validator key extraction has failed.
    ValidatorKeyExtraction = rpc_validator_key_logic::ApiErrorCode::ValidatorKeyExtraction as _,
}

pub mod app {
    //! Utility functions for producing jsonrpsee responses for the application level errors.

    use jsonrpsee::{
        core::Error,
        types::{error::CallError, ErrorObject},
    };
    use serde::Serialize;

    use super::ApiErrorCode;

    /// A simple error without the custom error data.
    pub fn simple(code: ApiErrorCode, message: impl Into<String>) -> Error {
        raw(code, message, Option::<()>::None)
    }

    /// An error with the custom error data.
    pub fn data<T: Serialize>(code: ApiErrorCode, message: impl Into<String>, data: T) -> Error {
        raw(code, message, Some(data))
    }

    /// A general form of an error with or without the custom error data.
    pub fn raw<T: Serialize>(
        code: ApiErrorCode,
        message: impl Into<String>,
        data: Option<T>,
    ) -> Error {
        let error_object = ErrorObject::owned(code as _, message, data);
        Error::Call(CallError::Custom(error_object))
    }
}
