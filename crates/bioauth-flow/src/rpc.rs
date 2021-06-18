//! RPC interface for the bioauth flow.

use jsonrpc_core::futures::Future;
use jsonrpc_core::Error as RpcError;
use jsonrpc_derive::rpc;

/// Future that resolves to account nonce.
pub type FutureResult<T> = Box<dyn Future<Item = T, Error = RpcError> + Send>;

/// The API exposed via JSON-RPC.
#[rpc]
pub trait BioauthApi<SessionToken> {
    /// Get a FaceTec Session Token.
    #[rpc(name = "bioauth_getFacetecSessionToken")]
    fn get_facetec_session_token(&self) -> FutureResult<SessionToken>;

    /// Provide the FaceTec FaceScan for the currently running enrollemnt or authentication process.
    #[rpc(name = "bioauth_provideFaceScan")]
    fn provide_facescan(&self) -> FutureResult<SessionToken>;
}
