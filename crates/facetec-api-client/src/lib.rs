//! Client API for the FaceTec Server SDK.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use reqwest::{RequestBuilder, Response};
use serde::de::DeserializeOwned;
use thiserror::Error;

pub mod db_enroll;
pub mod db_search;
pub mod enrollment3d;
pub mod response_body_error;
pub mod session_token;

mod types;

#[cfg(test)]
mod tests;

pub use response_body_error::ResponseBodyError;
pub use types::*;

/// The generic error type for the client calls.
#[derive(Error, Debug)]
pub enum Error<T: std::error::Error + 'static> {
    /// A call-specific error.
    #[error("server error: {0}")]
    Call(T),
    /// An error due to failure to load or parse the response body.
    #[error(transparent)]
    ResponseBody(#[from] ResponseBodyError),
    /// An error coming from the underlying reqwest layer.
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

/// The robonode client.
#[derive(Debug)]
pub struct Client<RBEI> {
    /// Underyling HTTP client used to execute network calls.
    pub reqwest: reqwest::Client,
    /// The base URL to use for the routes.
    pub base_url: String,
    /// The Device Key Identifier to pass in the header.
    pub device_key_identifier: String,
    /// The inspector for the response body.
    pub response_body_error_inspector: RBEI,
}

impl<RBEI> Client<RBEI> {
    /// Prepare the URL.
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Apply some common headers.
    fn apply_headers(&self, req: RequestBuilder) -> RequestBuilder {
        req.header("X-Device-Key", self.device_key_identifier.clone())
    }

    /// An internal utility to prepare an HTTP request.
    /// Applies some common logic.
    fn build<F>(&self, path: &str, f: F) -> RequestBuilder
    where
        F: FnOnce(String) -> RequestBuilder,
    {
        let url = self.build_url(path);
        self.apply_headers(f(url))
    }

    /// An internal utility to prepare a GET HTTP request.
    /// Applies some common logic.
    fn build_get(&self, path: &str) -> RequestBuilder {
        self.build(path, |url| self.reqwest.get(url))
    }

    /// An internal utility to prepare a POST HTTP request.
    /// Applies some common logic.
    fn build_post<T>(&self, path: &str, body: &T) -> RequestBuilder
    where
        T: serde::Serialize + ?Sized,
    {
        self.build(path, |url| self.reqwest.post(url)).json(body)
    }
}

impl<RBEI> Client<RBEI>
where
    RBEI: response_body_error::Inspector,
{
    /// A custom JSON parsing logic for more control over how we handle JSON parsing errors.
    async fn parse_json<T>(&self, res: Response) -> Result<T, ResponseBodyError>
    where
        T: DeserializeOwned,
    {
        let full = res.bytes().await.map_err(ResponseBodyError::BodyRead)?;

        match serde_json::from_slice(&full) {
            Ok(val) => Ok(val),
            Err(err) => {
                let err = ResponseBodyError::Json {
                    source: err,
                    body: full,
                };
                self.response_body_error_inspector.inspect(&err).await;
                Err(err)
            }
        }
    }
}
