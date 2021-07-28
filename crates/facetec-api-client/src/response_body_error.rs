//! Response body error.

use thiserror::Error;

/// An error while loading or parsing the response body.
#[derive(Error, Debug)]
pub enum ResponseBodyError {
    /// Unable to read the response body, probably due to a socket failure of some kind.
    #[error("response body reading error: {0}")]
    BodyRead(#[source] reqwest::Error),
    /// Unable to parse the JSON response. Might be because the response is not in JSON when we
    /// expected it to be in JSON, or if the JSON that we got does not match the definition that
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

/// An interface to allow inspection of the response body errors.
#[async_trait::async_trait]
pub trait Inspector {
    /// Invoked when we're reading the raw bytes, before parsing.
    async fn inspect_raw(&self, bytes: &[u8]);

    /// Invoked when the response body error occurs.
    async fn inspect_error(&self, error: &ResponseBodyError);
}

/// An inspector that does nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopInspector;

#[async_trait::async_trait]
impl Inspector for NoopInspector {
    async fn inspect_raw(&self, _bytes: &[u8]) {
        // do nothing
    }

    async fn inspect_error(&self, _error: &ResponseBodyError) {
        // do nothing
    }
}
