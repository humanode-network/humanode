//! Client API for the Humanode Biometrics Server.

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{error_response::ErrorResponse, Client, Error, ErrorChain, StepRequest, StepResponse};

impl Client {
    /// Perform the enroll-step call to the server.
    pub async fn enroll_step(
        &self,
        req: StepRequest<'_>,
    ) -> Result<EnrollStepResponse, Error<EnrollStepError>> {
        let url = format!("{}/enroll-step", self.base_url);
        let res = self.reqwest.post(url).json(&req).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            status => Err(Error::Call(EnrollStepError::from_response(
                status,
                res.text().await?,
            ))),
        }
    }
}

/// The data of the enroll-step response.
pub type EnrollStepResponse = StepResponse<EnrollDone>;

/// The data of a done enroll flow.
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct EnrollDone {}

/// The enroll-step-specific error condition.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum EnrollStepError {
    /// The public key is invalid.
    #[error("invalid public key")]
    InvalidPublicKey(ErrorChain),

    /// The face scan was rejected.
    #[error("face scan rejected")]
    FaceScanRejected(ErrorChain),

    /// The public key is already used.
    #[error("public key already used")]
    PublicKeyAlreadyUsed(ErrorChain),

    /// The person is already enrolled.
    #[error("person already enrolled")]
    PersonAlreadyEnrolled(ErrorChain),

    /// A logic internal error occurred on the server end.
    #[error("logic internal error")]
    LogicInternal(ErrorChain),

    /// The signature was invalid.
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

impl EnrollStepError {
    /// Parse the error response.
    fn from_response(_status: StatusCode, body: String) -> Self {
        let (error_code, chain) = match body.try_into() {
            Ok(ErrorResponse { error_code, chain }) => (error_code, chain),
            Err(body) => return Self::Unknown(body),
        };
        match error_code.as_str() {
            "ENROLL_INVALID_PUBLIC_KEY" => Self::InvalidPublicKey(chain),
            "ENROLL_FACE_SCAN_REJECTED" => Self::FaceScanRejected(chain),
            "ENROLL_PUBLIC_KEY_ALREADY_USED" => Self::PublicKeyAlreadyUsed(chain),
            "ENROLL_PERSON_ALREADY_ENROLLED" => Self::PersonAlreadyEnrolled(chain),
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
