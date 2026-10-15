//! Error response handling logic.

use serde::Deserialize;

use crate::{ErrorChain, ResponseBlob, StepChainLink};

/// A utility type assisting with decoding error response bodies.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponseBody {
    /// A machine-readable code identifying the error.
    error_code: String,

    /// The response blob to pass to the FaceTec SDK.
    response_blob: Option<ResponseBlob>,

    /// The link to continue the chain with.
    chain: Option<StepChainLink>,
}

/// A decoded error response.
#[derive(Debug)]
pub(super) struct ErrorResponse {
    /// A machine-readable code identifying the error.
    pub error_code: String,

    /// What the server sent along with the error.
    pub chain: ErrorChain,
}

impl TryFrom<String> for ErrorResponse {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let ErrorResponseBody {
            error_code,
            response_blob,
            chain,
        } = match serde_json::from_str(&s) {
            Ok(body) => body,
            Err(_parsing_error) => return Err(s),
        };

        // The response blob and the chain come together or not at all.
        let chain = match (response_blob, chain) {
            (None, None) => ErrorChain::Closed,
            (Some(response_blob), Some(chain)) => ErrorChain::Open {
                response_blob,
                chain,
            },
            _ => return Err(s),
        };

        Ok(Self { error_code, chain })
    }
}

#[cfg(test)]
mod tests;
