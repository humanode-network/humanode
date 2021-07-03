//! An [`ft::response_body_error::Inspector`] that will log errors.

use facetec_api_client as ft;
use tracing::error;

/// An inspector that will log the errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct LoggingInspector;

#[async_trait::async_trait]
impl ft::response_body_error::Inspector for LoggingInspector {
    async fn inspect(&self, err: &ft::ResponseBodyError) {
        match err {
            ft::ResponseBodyError::Json { source, body } => error!(
                message = "FaceTec API failed to parse JSON response",
                error = %source,
                body = ?body,
            ),
            err => error!(
                message = "FaceTec API failed to parse response body",
                error = %err,
            ),
        }
    }
}
