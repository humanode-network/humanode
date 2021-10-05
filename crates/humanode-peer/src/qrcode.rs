//! QR Code generation.

use std::borrow::Cow;

use tracing::{error, info};
use url::Url;

/// The information necessary for printing the Web App QR Code.
pub struct WebApp {
    /// The Web App URL.
    url: Url,
}

impl WebApp {
    /// Create a new [`WebApp`] and validate that the resulting URL is valid.
    pub fn new(
        base_url: impl AsRef<str>,
        rpc_url: impl AsRef<str>,
    ) -> Result<Self, Cow<'static, str>> {
        let mut url = Url::parse(base_url.as_ref()).map_err(|err| err.to_string())?;
        url.path_segments_mut()
            .map_err(|()| Cow::Borrowed("invalid web app URL"))?
            .push("open");
        url.query_pairs_mut().append_pair("url", rpc_url.as_ref());
        Ok(Self { url })
    }

    /// Provide the Web App URL.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// Print the QR Code for the Web App to the terminal.
    pub fn print(&self) {
        info!("Please visit {} to proceed", self.url);
        qr2term::print_qr(self.url.as_str())
            .unwrap_or_else(|error| error!(message = "Failed to generate QR Code", %error));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construction() {
        let webapp = WebApp::new("https://example.com", "http://localhost:9933").unwrap();

        assert_eq!(
            webapp.url,
            Url::parse("https://example.com/open?url=http%3A%2F%2Flocalhost%3A9933").unwrap()
        );
    }
}
