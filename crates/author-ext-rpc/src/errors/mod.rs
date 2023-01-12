//! All humanode related error kinds that we expose in the RPC.

pub mod runtime_api;
pub mod tx_pool;
pub mod validator;

pub use runtime_api::*;
pub use tx_pool::*;
pub use validator::*;

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
pub enum ApiErrorCode {
    /// Call to runtime api has failed.
    RuntimeApi = 300,
    /// Authenticate transaction has failed.
    Transaction = 400,
    /// Validator key is not available.
    MissingValidatorKey = 500,
    /// Validator key extraction has failed.
    ValidatorKeyExtraction = 600,
}
