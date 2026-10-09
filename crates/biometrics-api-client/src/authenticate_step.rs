//! Client API for the Humanode Biometrics Server.

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{error_response::ErrorResponse, Client, Error, ErrorChain, StepRequest, StepResponse};

impl Client {
    /// Perform the authenticate-step call to the server.
    pub async fn authenticate_step(
        &self,
        req: StepRequest<'_>,
    ) -> Result<AuthenticateStepResponse, Error<AuthenticateStepError>> {
        let url = format!("{}/authenticate-step", self.base_url);
        let res = self.reqwest.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            status => Err(Error::Call(AuthenticateStepError::from_response(
                status,
                res.text().await?,
            ))),
        }
    }
}

/// The data of the authenticate-step response.
pub type AuthenticateStepResponse = StepResponse<AuthenticateDone>;

/// The data of a done authenticate flow.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateDone {
    /// An opaque auth ticket generated for this authentication attempt.
    #[serde(with = "crate::hex_bytes")]
    pub auth_ticket: Box<[u8]>,

    /// The server signature for this opaque auth ticket.
    #[serde(with = "crate::hex_bytes")]
    pub auth_ticket_signature: Box<[u8]>,
}

/// The authenticate-step-specific error condition.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AuthenticateStepError {
    /// The person was not found, it is likely because they haven't enrolled first.
    #[error("person not found")]
    PersonNotFound(ErrorChain),

    /// The face scan was rejected, this is likely due to a failed liveness check.
    #[error("face scan rejected")]
    FaceScanRejected(ErrorChain),

    /// A logic internal error occurred on the server end.
    #[error("logic internal error")]
    LogicInternal(ErrorChain),

    /// The signature was invalid, which means that the validator private key used for signing and
    /// the public key that the person enrolled with don't match.
    #[error("signature invalid")]
    SignatureInvalid,

    /// The chain was invalid.
    #[error("chain invalid")]
    ChainInvalid,

    /// The chain is already done.
    #[error("chain done")]
    ChainDone,

    /// The nonce was invalid.
    #[error("nonce invalid")]
    NonceInvalid,

    /// An error with an unknown code occurred.
    #[error("unknown error code: {0}")]
    UnknownCode(String, ErrorChain),

    /// Some other error occurred.
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl AuthenticateStepError {
    /// Parse the error response.
    fn from_response(_status: StatusCode, body: String) -> Self {
        let (error_code, chain) = match body.try_into() {
            Ok(ErrorResponse { error_code, chain }) => (error_code, chain),
            Err(body) => return Self::Unknown(body),
        };
        match error_code.as_str() {
            "AUTHENTICATE_PERSON_NOT_FOUND" => Self::PersonNotFound(chain),
            "AUTHENTICATE_FACE_SCAN_REJECTED" => Self::FaceScanRejected(chain),
            "LOGIC_INTERNAL_ERROR" => Self::LogicInternal(chain),
            "SIGNATURE_INVALID" => Self::SignatureInvalid,
            "CHAIN_INVALID" => Self::ChainInvalid,
            "CHAIN_DONE" => Self::ChainDone,
            "NONCE_INVALID" => Self::NonceInvalid,
            _ => Self::UnknownCode(error_code, chain),
        }
    }
}

#[cfg(test)]
mod tests;
