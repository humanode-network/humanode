//! The validator key integration logic.

use std::convert::Infallible;

use bioauth_flow::flow::Signer;
use sp_core::sr25519;
use sp_core::Pair;

/// Validator key is for the purposes of using it with the
/// bioauth enroll and authenticate.
#[derive(Clone)]
pub struct ValidatorKey {
    /// Validator's key pair.
    pub pair: sr25519::Pair,
    /// Validator's public key.
    pub public: String,
}

impl ValidatorKey {
    /// A constructor.
    pub fn new(key_seed: &str) -> Result<Self, String> {
        let pair =
            sr25519::Pair::from_string(key_seed, None).map_err(|_| "Invalid seed".to_owned())?;
        let public = pair.public().to_string();
        Ok(ValidatorKey { pair, public })
    }
}

#[async_trait::async_trait]
impl Signer<Vec<u8>> for ValidatorKey {
    type Error = Infallible;

    async fn sign<'a, D>(&self, data: D) -> Result<Vec<u8>, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        let signature = self.pair.sign(data.as_ref());
        Ok(signature.0.to_vec())
    }
}

impl AsRef<[u8]> for ValidatorKey {
    fn as_ref(&self) -> &[u8] {
        self.public.as_bytes()
    }
}
