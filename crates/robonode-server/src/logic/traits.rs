//! The logic-related traits.

/// Signer provides signatures for the data.
#[async_trait::async_trait]
pub trait Signer<S> {
    /// Signature error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Sign the provided data and return the signature, or an error if the siging fails.
    async fn sign<'a, D>(&self, data: D) -> Result<S, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// Verifier provides the verification of the data accompanied with the
/// signature or proof data.
#[async_trait::async_trait]
pub trait Verifier<S: ?Sized> {
    /// Verification error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Verify that provided data is indeed correctly signed with the provided
    /// signature.
    async fn verify<'a, D>(&self, data: D, signature: S) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// The trait to make logiс operations.
#[async_trait::async_trait]
pub trait LogicOp<Request> {
    /// Logic operation Response type.
    type Response;
    /// Logic operation Error type.
    type Error;

    /// Process logic operation request.
    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error>;
}

/// Public key provider, intended to return the public key of the robonode itself.
pub trait PublicKeyProvider {
    /// Provide the public key.
    fn public_key(&self) -> &[u8];
}
