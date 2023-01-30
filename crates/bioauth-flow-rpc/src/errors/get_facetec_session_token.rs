//! The `get_facetec_session_token` method error.

use super::{app, ApiErrorCode};

/// The `get_facetec_session_token` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during doing a call into robonode.
    Robonode(robonode_client::Error<robonode_client::GetFacetecSessionTokenError>),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Robonode(err) => app::simple(ApiErrorCode::Robonode, err.to_string()),
        }
    }
}
