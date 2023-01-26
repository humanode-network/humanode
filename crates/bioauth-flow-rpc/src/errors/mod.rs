//! All humanode related error kinds that we expose in the RPC.

pub mod robonode;
pub mod runtime_api;
pub mod signer;
pub mod tx_pool;

pub use robonode::*;
pub use runtime_api::*;
pub use signer::*;
pub use tx_pool::*;

/// Custom rpc error codes.
#[derive(Debug, Clone, Copy)]
pub enum ApiErrorCode {
    /// Signer has failed.
    Signer = 100,
    /// Request to robonode has failed.
    Robonode = 200,
    /// Call to runtime api has failed.
    RuntimeApi = 300,
    /// Authenticate transaction has failed.
    Transaction = 400,
}