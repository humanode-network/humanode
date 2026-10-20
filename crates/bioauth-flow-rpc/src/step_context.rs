//! The step flow context, passed through the web app between the steps.

use codec::{Decode, DecodeAll, Encode};

/// The step flow context.
///
/// Holds the chain link the biometrics server issued at a step. The peer hands it to the web app
/// in the SCALE encoding, and decodes it back at the next step. The web app treats it as opaque.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub struct StepContext {
    /// The chain nonce.
    pub nonce: u64,

    /// The sealed chain state.
    pub ciphertext: Vec<u8>,
}

impl StepContext {
    /// Encodes the context into the bytes passed to the web app.
    pub fn to_bytes(&self) -> sp_core::Bytes {
        self.encode().into()
    }

    /// Decodes the context from the bytes passed by the web app.
    ///
    /// Trailing bytes are rejected.
    pub fn from_bytes(bytes: &sp_core::Bytes) -> Result<Self, codec::Error> {
        Self::decode_all(&mut &bytes[..])
    }

    /// Advances the context to the next step: the nonce is incremented, the ciphertext is kept.
    ///
    /// `None` when the nonce would overflow.
    pub fn next(self) -> Option<Self> {
        Some(Self {
            nonce: self.nonce.checked_add(1)?,
            ciphertext: self.ciphertext,
        })
    }
}

impl From<&biometrics_api_client::StepChainLink> for StepContext {
    fn from(link: &biometrics_api_client::StepChainLink) -> Self {
        Self {
            nonce: link.nonce,
            ciphertext: link.ciphertext.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests;
