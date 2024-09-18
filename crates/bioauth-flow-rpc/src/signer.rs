//! Signer.

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

/// A factory that spits out [`Signer`]s.
pub trait Factory<S, K> {
    /// The type of [`Signer`] this factory will create.
    type Signer: Signer<S>;

    /// Create a [`Signer`] using the provided public key.
    fn new_signer(&self, key: K) -> Self::Signer;
}

impl<S, T, F, K, P> Factory<T, K> for P
where
    P: std::ops::Deref<Target = F>,
    F: Fn(K) -> S,
    S: Signer<T>,
{
    type Signer = S;

    fn new_signer(&self, key: K) -> Self::Signer {
        self(key)
    }
}
