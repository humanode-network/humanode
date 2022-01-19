//! The bioauth flow implementation, aka the logic for communication between the humanode
//! (aka humanode-peer), the app on the handheld device that perform that biometric capture,
//! and the robonode server that's responsible for authenticating against the bioauth system.

pub mod flow;
pub mod rpc;

/// Signer provides signatures for the data.
#[async_trait::async_trait]
pub trait Signer<S> {
    /// Signature error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Sign the provided data and return the signature, or an error if the signing fails.
    async fn sign<'a, D>(&self, data: D) -> std::result::Result<S, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// Interface for calling transactions.
#[async_trait::async_trait]
pub trait TransactionManager {
    /// Transaction error.
    type Error;

    /// Submit an authenticate transaction.
    async fn submit_authenticate<OpaqueAuthTicket, Commitment>(
        &self,
        auth_ticket: OpaqueAuthTicket,
        ticket_signature: Commitment,
    ) -> Result<(), Self::Error>
    where
        OpaqueAuthTicket: AsRef<[u8]> + Send + Sync,
        Commitment: AsRef<[u8]> + Send + Sync;
}
