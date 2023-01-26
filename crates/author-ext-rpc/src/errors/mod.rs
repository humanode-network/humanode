//! All author extension related error kinds that we expose in the RPC.

pub mod get_validator_public_key;
pub mod set_keys;
pub mod tx_pool;

pub use get_validator_public_key::*;
pub use set_keys::*;
pub use tx_pool::*;

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
pub enum ApiErrorCode {
    /// Call to runtime api has failed.
    RuntimeApi = 300,
    /// Authenticate transaction has failed.
    Transaction = 400,
}
