//! The peer side data of a step request.

use crate::step_context::StepContext;

/// The peer side data of a step request, signed and ready to be sent.
pub struct SignedStep<PublicKey> {
    /// The validator public key the chain is bound to.
    pub public_key: PublicKey,

    /// The context of the step, with the nonce already advanced; absent for the first step.
    pub context: Option<StepContext>,

    /// The signature over the step signing payload.
    pub signature: Vec<u8>,
}

impl<PublicKey: AsRef<[u8]>> SignedStep<PublicKey> {
    /// Builds the request to send to the biometrics API.
    pub fn request<'a>(&'a self, request_blob: &'a str) -> biometrics_api_client::StepRequest<'a> {
        biometrics_api_client::StepRequest {
            request_blob,
            chain: step_chain(self.public_key.as_ref(), self.context.as_ref()),
            signature: &self.signature,
        }
    }
}

/// Builds the chain for a step: a new chain bound to the public key when there is no context,
/// the continuation of the chain from the context otherwise.
pub fn step_chain<'a>(
    public_key: &'a [u8],
    context: Option<&'a StepContext>,
) -> biometrics_api_client::StepChain<'a> {
    match context {
        None => biometrics_api_client::StepChain::Start { public_key },
        Some(context) => biometrics_api_client::StepChain::Next {
            nonce: context.nonce,
            ciphertext: &context.ciphertext,
        },
    }
}
