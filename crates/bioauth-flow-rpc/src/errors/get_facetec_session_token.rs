//! The `get_facetec_session_token` method error kinds.

use jsonrpsee::{
    core::Error as JsonRpseeError,
    types::error::{CallError, ErrorCode, ErrorObject},
};

use super::ApiErrorCode;

/// The `get_facetec_session_token` method error kinds.
#[derive(Debug)]
pub enum GetFacetecSessionToken<T: std::error::Error + 'static> {
    /// An error that can occur during doing a call into robonode.
    Robonode(robonode_client::Error<T>),
}

impl<T: std::error::Error + 'static> From<GetFacetecSessionToken<T>> for JsonRpseeError {
    fn from(err: GetFacetecSessionToken<T>) -> Self {
        match err {
            GetFacetecSessionToken::Robonode(err) => {
                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::ServerError(ApiErrorCode::Robonode as _).code(),
                    err.to_string(),
                    None::<()>,
                )))
            }
        }
    }
}
