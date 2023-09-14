//! The ngrok API client.

use url::Url;

use crate::data::response;

/// HTTP client params.
pub struct Client {
    /// Underlying HTTP client used to execute network calls.
    reqwest: reqwest::Client,
    /// The URL to use as a base.
    base_url: Url,
}

/// The error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The base URL is invalid and can't be a base.
    #[error("base url is invalid as it cannot be a base")]
    BaseUrlCannotBeABase,
    /// Error in the transport layer.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Error while checking the response status code.
    #[error("bad status code: {0}")]
    BadStatus(http::StatusCode),
}

/// The base URL is invalid and can't be a base.
#[derive(Debug, thiserror::Error)]
#[error("base url is invalid as it cannot be a base")]
pub struct BaseUrlCannotBeABaseError;

impl Client {
    /// The standard base URL.
    pub fn standard_base_url() -> Url {
        Url::parse("http://localhost:4040/api").expect("static value should always work")
    }

    /// Create a new [`Client`].
    pub fn new(reqwest: reqwest::Client, base_url: Url) -> Result<Self, BaseUrlCannotBeABaseError> {
        if base_url.cannot_be_a_base() {
            return Err(BaseUrlCannotBeABaseError);
        }
        Ok(Self { reqwest, base_url })
    }

    /// Do an HTTP request and obtain an HTTP response.
    pub async fn call<Request>(
        &self,
        req: &Request,
        params: Request::Params,
    ) -> Result<Request::Response, Error>
    where
        Request: crate::http::Definition,
    {
        let url = self.build_url::<Request>(params)?;
        let res = self
            .reqwest
            .request(Request::METHOD, url)
            .json(req)
            .send()
            .await?;
        self.check_response(&res)?;
        let res: response::Envelope<Request::Response> = res.json().await?;
        Ok(res.payload)
    }

    /// Build a full request URL.
    fn build_url<Request>(&self, params: Request::Params) -> Result<Url, Error>
    where
        Request: crate::http::Definition,
    {
        let mut url = self.base_url.clone();

        {
            let mut path_segments = url
                .path_segments_mut()
                .map_err(|_| Error::BaseUrlCannotBeABase)?;
            path_segments.extend(Request::path_segments(params));
        }

        Ok(url)
    }

    /// Check the response.
    fn check_response(&self, res: &reqwest::Response) -> Result<(), Error> {
        let status = res.status();
        if !status.is_success() {
            return Err(Error::BadStatus(status));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_base_url() {
        let base_url = Client::standard_base_url();
        Client::new(Default::default(), base_url).unwrap();
    }
}
