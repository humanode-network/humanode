//! An [`ft::response_body_error::Inspector`] that will log errors.

use facetec_api_client as ft;
use tracing::{error, trace};

/// An inspector that will log the errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct LoggingInspector;

#[async_trait::async_trait]
impl ft::response_body_error::Inspector for LoggingInspector {
    async fn inspect_raw(&self, bytes: &[u8]) {
        let body = String::from_utf8_lossy(bytes);
        trace!(message = "FaceTec API response obtained", %body);
    }

    async fn inspect_error(&self, err: &ft::ResponseBodyError) {
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
