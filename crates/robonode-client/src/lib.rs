//! Client API for the Humanode's Bioauth Robonode.

use thiserror::Error;

mod authenticate;
mod enroll;
mod get_facetec_device_sdk_params;
mod get_facetec_session_token;

pub use authenticate::*;
pub use enroll::*;
pub use get_facetec_device_sdk_params::*;
pub use get_facetec_session_token::*;

/// The generic error type for the client calls.
#[derive(Error, Debug)]
pub enum Error<T: std::error::Error + 'static> {
    /// A call-specific error.
    #[error("server error: {0}")]
    Call(T),
    /// An error coming from the underlying reqwest layer.
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

/// The robonode client.
#[derive(Debug)]
pub struct Client {
    /// Underyling HTTP client used to execute network calls.
    pub reqwest: reqwest::Client,
    /// The base URL to use for the routes.
    pub base_url: String,
}
