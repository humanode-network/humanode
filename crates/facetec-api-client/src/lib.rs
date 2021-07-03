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

mod db_enroll;
mod db_search;
mod enrollment3d;
mod session_token;
mod types;

pub use db_enroll::*;
pub use db_search::*;
pub use enrollment3d::*;
pub use session_token::*;
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

/// An error while loading or parsing the response body.
#[derive(Error, Debug)]
pub enum ResponseBodyError {
    /// Unable to read the response body, probably due to a socket failure of some kind.
    #[error("response body reading error: {0}")]
    BodyRead(#[source] reqwest::Error),
    /// Unable to parse the JSON response. Might be because the response is not in JSON when we
    /// expected it to be in JSON, or if the JSON the we got does not match the definition that
    /// serde expects on our end.
    #[error("JSON response parsing error: {source}")]
    Json {
        /// The underlying [`serde_json::Error`] error.
        #[source]
        source: serde_json::Error,
        /// The full response body that caused this error, useful for inspection.
        body: bytes::Bytes,
    },
}

/// The robonode client.
#[derive(Debug)]
pub struct Client {
    /// Underyling HTTP client used to execute network calls.
    pub reqwest: reqwest::Client,
    /// The base URL to use for the routes.
    pub base_url: String,
    /// The Device Key Identifier to pass in the header.
    pub device_key_identifier: String,
}

impl Client {
    /// Prepare the URL.
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Apply some common headers.
    fn apply_headers(&self, req: RequestBuilder) -> RequestBuilder {
        req.header("X-Device-Key", self.device_key_identifier.clone())
    }

    /// An internal utility to prepare a GET HTTP request.
    /// Applies some common logic.
    fn build_get(&self, path: &str) -> RequestBuilder {
        let url = self.build_url(path);
        self.apply_headers(self.reqwest.get(url))
    }

    /// An internal utility to prepare a POST HTTP request.
    /// Applies some common logic.
    fn build_post<T>(&self, path: &str, body: &T) -> RequestBuilder
    where
        T: serde::Serialize + ?Sized,
    {
        let url = self.build_url(path);
        self.apply_headers(self.reqwest.post(url)).json(body)
    }

    /// A custom JSON parsing logic for more control over how we handle JSON parsing errors.
    async fn parse_json<T>(&self, res: Response) -> Result<T, ResponseBodyError>
    where
        T: DeserializeOwned,
    {
        let full = res.bytes().await.map_err(ResponseBodyError::BodyRead)?;

        serde_json::from_slice(&full).map_err(|err| ResponseBodyError::Json {
            source: err,
            body: full,
        })
    }
}
