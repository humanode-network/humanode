//! Client API for the FaceTec Server SDK.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use facetec_response::FacetecResponse;
use reqwest::{RequestBuilder, Response};
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

pub mod db_delete;
pub mod db_enroll;
pub mod db_search;
pub mod enrollment3d;
pub mod facetec_response;
pub mod reset;
pub mod response_body_error;
pub mod session_token;

mod types;

#[cfg(test)]
mod tests;

pub use response_body_error::ResponseBodyError;
pub use types::*;

/// The generic error type for the client calls.
#[derive(Error, Debug)]
pub enum Error {
    /// A server error.
    #[error(transparent)]
    Server(#[from] ServerError),
    /// An error due to failure to load or parse the response body.
    #[error(transparent)]
    ResponseBody(#[from] ResponseBodyError),
    /// An error coming from the underlying reqwest layer.
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

/// An error response originating from the FaceTec Server itself.
#[derive(Error, Debug, Deserialize)]
#[error("server error: {error_message}")]
pub struct ServerError {
    /// A human-readable message characterizing the error.
    #[serde(rename = "errorMessage")]
    pub error_message: String,
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
    /// The fake IP address to inject via `X-FT-IPAddress` header.
    pub injected_ip_address: Option<String>,
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
        let req = req.header("X-Device-Key", self.device_key_identifier.clone());

        if let Some(ref injected_ip_address) = self.injected_ip_address {
            req.header("X-FT-IPAddress", injected_ip_address.clone())
        } else {
            req
        }
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

        self.response_body_error_inspector.inspect_raw(&full).await;

        match serde_json::from_slice(&full) {
            Ok(val) => Ok(val),
            Err(err) => {
                let err = ResponseBodyError::Json {
                    source: err,
                    body: full,
                };
                self.response_body_error_inspector.inspect_error(&err).await;
                Err(err)
            }
        }
    }

    /// Parse a FaceTec Server response or generate a parsing error.
    async fn parse_response<T>(&self, res: Response) -> Result<T, crate::Error>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let body: FacetecResponse<T> = self.parse_json(res).await?;
        Ok(body.into_inner()?)
    }
}
