//! All bioauth flow error kinds that we expose in the RPC.

pub mod authenticate;
pub mod enroll;
pub mod get_facetec_device_sdk_params;
pub mod get_facetec_session_token;
pub mod robonode;
pub mod sign;
pub mod status;
pub mod tx_pool;

pub use authenticate::*;
pub use enroll::*;
pub use get_facetec_device_sdk_params::*;
pub use get_facetec_session_token::*;
pub use robonode::*;
pub use sign::*;
pub use status::*;
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
