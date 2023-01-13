//! All humanode related error kinds that we expose in the RPC.

pub mod runtime_api;
pub mod tx_pool;

pub use runtime_api::*;
pub use tx_pool::*;

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
pub enum ApiErrorCode {
    /// Call to runtime api has failed.
    RuntimeApi = 300,
    /// Authenticate transaction has failed.
    Transaction = 400,
}
