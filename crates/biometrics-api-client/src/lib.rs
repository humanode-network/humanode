//! Client API for the Humanode Biometrics Server.

use thiserror::Error;

mod authenticate_step;
mod enroll_step;
mod error_response;
mod get_facetec_device_sdk_params;
mod step;
#[cfg(test)]
mod test_utils;

pub use authenticate_step::*;
pub use enroll_step::*;
pub use get_facetec_device_sdk_params::*;
pub use step::*;

/// A type alias representing a response blob.
pub type ResponseBlob = String;

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

/// The biometrics server client.
#[derive(Debug)]
pub struct Client {
    /// Underlying HTTP client used to execute network calls.
    pub reqwest: reqwest::Client,

    /// The base URL to use for the routes.
    pub base_url: String,
}
