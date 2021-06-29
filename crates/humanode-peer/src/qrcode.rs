//! QR Code generation.

use tracing::{error, info};
use url::Url;

/// The information necessary for printing the Web App QR Code.
pub struct WebApp {
    /// The Web App URL.
    url: Url,
}

impl WebApp {
    /// Create a new [`WebApp`] and validate that the resulting URL is valid.
    pub fn new(base_url: &str, rpc_url: &str) -> Result<Self, String> {
        let mut url = Url::parse(base_url).map_err(|err| err.to_string())?;
        url.path_segments_mut()
            .map_err(|_| "invalid base URL".to_owned())?
            .push("humanode")
            .push(rpc_url);
        Ok(Self { url })
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
            Url::parse("https://example.com/humanode/http:%2F%2Flocalhost:9933").unwrap()
        );
    }
}
