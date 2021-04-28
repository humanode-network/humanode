//! Client API for the Humanode's Bioauth Robonode.

#![deny(missing_docs, clippy::missing_docs_in_private_items)]

use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

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

/// The robonode client.
#[derive(Debug)]
pub struct Client {
    /// Underyling HTTP client used to execute network calls.
    pub reqwest: reqwest::Client,
    /// The base URL to use for the routes.
    pub base_url: String,
}

impl Client {
    /// Perform the enroll call to the server.
    pub async fn enroll(&self, req: EnrollRequest<'_>) -> Result<(), Error<EnrollError>> {
        let url = format!("{}/enroll", self.base_url);
        let res = self.reqwest.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::CREATED => Ok(()),
            StatusCode::CONFLICT => Err(Error::Call(EnrollError::AlreadyEnrolled)),
            _ => Err(Error::Call(EnrollError::Unknown(res.text().await?))),
        }
    }

    /// Perform the authenticate call to the server.
    pub async fn authenticate(
        &self,
        req: AuthenticateRequest<'_>,
    ) -> Result<(), Error<AuthenticateError>> {
        let url = format!("{}/authenticate", self.base_url);
        let res = self.reqwest.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            StatusCode::NOT_FOUND => Err(Error::Call(AuthenticateError::MatchNotFound)),
            _ => Err(Error::Call(AuthenticateError::Unknown(res.text().await?))),
        }
    }
}

/// Input data for the enroll request.
#[derive(Debug, Serialize)]
pub struct EnrollRequest<'a> {
    /// The public key to be used as an identity.
    public_key: &'a [u8],
    /// The FaceTec 3D FaceScan to associate with the identity.
    face_scan: &'a [u8],
}

/// The enroll-specific error condition.
#[derive(Error, Debug)]
pub enum EnrollError {
    /// The face scan or public key were already enrolled.
    #[error("already enrolled")]
    AlreadyEnrolled,
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}

/// Input data for the authenticate request.
#[derive(Debug, Serialize)]
pub struct AuthenticateRequest<'a> {
    /// The FaceTec 3D FaceScan to associate with the identity.
    face_scan: &'a [u8],
    /// The signature of the FaceTec 3D FaceScan, proving the posession of the
    /// private key by the issuer of this request.
    face_scan_signature: &'a [u8],
}

/// Input data for the authenticate request.
#[derive(Debug, Serialize)]
pub struct AuthenticateResponse {
    /// The public key that matched with the provided FaceTec 3D FaceScan.
    public_key: Box<[u8]>,
    /// The robonode signatire for this public key.
    // TODO: we need a nonce to prevent replay attack, don't we?
    public_key_signature: Box<[u8]>,
}

/// The authenticate-specific error condition.
#[derive(Error, Debug)]
pub enum AuthenticateError {
    /// The match was not found, user likely needs to register first, or retry
    /// with another face scan.
    #[error("match not found")]
    MatchNotFound,
    /// Some other error occured.
    #[error("unknown error: {0}")]
    Unknown(String),
}
