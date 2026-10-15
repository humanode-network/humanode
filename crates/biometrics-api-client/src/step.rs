//! The types shared by the step calls.

use codec::Encode;
use serde::{Deserialize, Serialize};

use crate::ResponseBlob;

/// A link of the chain, as issued by the server after a step.
///
/// The chain is the server's AEAD state: the nonce is the public part, the ciphertext is
/// the sealed part, opaque to the client.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepChainLink {
    /// The chain nonce.
    pub nonce: u64,

    /// The sealed chain state.
    pub ciphertext: Box<[u8]>,
}

/// The chain as sent with a step request.
#[derive(Debug, PartialEq, Eq, Serialize, Encode)]
#[serde(tag = "tag", rename_all = "camelCase")]
pub enum StepChain<'a> {
    /// Start a new chain.
    #[serde(rename_all = "camelCase")]
    Start {
        /// The public key the chain is bound to.
        public_key: &'a [u8],
    },

    /// Continue the chain from the link issued by the server at the previous step.
    #[serde(rename_all = "camelCase")]
    Next {
        /// The nonce of the previous link, incremented.
        nonce: u64,

        /// The ciphertext of the previous link, unchanged.
        ciphertext: &'a [u8],
    },
}

/// Input data for a step request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepRequest<'a> {
    /// The FaceTec SDK request blob.
    pub request_blob: &'a str,

    /// The chain this step belongs to.
    pub chain: StepChain<'a>,

    /// The signature of the [`StepSigningPayload`] with the private key of the node.
    /// Proves the possession of the private key by the issuer of this request.
    pub signature: &'a [u8],
}

/// The data a step request signature covers; signed in the SCALE encoding.
#[derive(Debug, Encode)]
pub struct StepSigningPayload<'a> {
    /// The FaceTec SDK request blob.
    pub request_blob: &'a str,

    /// The chain this step belongs to.
    pub chain: &'a StepChain<'a>,
}

/// The data of a step response.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepResponse<Done> {
    /// The response blob to pass to the FaceTec SDK.
    pub response_blob: ResponseBlob,

    /// Whether the flow continues or is done.
    #[serde(flatten)]
    pub status: StepStatus<Done>,
}

/// The flow status after a step.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum StepStatus<Done> {
    /// More steps are needed.
    Continue {
        /// The link to continue the chain with.
        chain: StepChainLink,
    },

    /// The flow is complete.
    Done(Done),
}

/// What the server sent along with an error.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorChain {
    /// The chain is closed, no further steps are accepted.
    Closed,

    /// The chain continues.
    Open {
        /// The response blob to pass to the FaceTec SDK.
        response_blob: ResponseBlob,

        /// The link to continue the chain with.
        chain: StepChainLink,
    },
}

#[cfg(test)]
mod tests;
