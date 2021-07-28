//! The validator key integration logic.

use std::convert::TryFrom;

use sp_application_crypto::Public;
use sp_application_crypto::RuntimePublic;

use crate::logic;

/// A validator public key implemented via substrate's aura consensus key.
pub struct AuraPublic(sp_application_crypto::sr25519::Public);

#[async_trait::async_trait]
impl logic::Verifier<Vec<u8>> for AuraPublic {
    type Error = ();

    async fn verify<'a, D>(&self, data: D, signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        let data = data.as_ref();
        let signature = sp_application_crypto::sr25519::Signature::try_from(signature.as_slice())?;
        Ok(self.0.verify(&data, &signature))
    }
}

impl TryFrom<&[u8]> for AuraPublic {
    type Error = ();

    fn try_from(val: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(sp_application_crypto::sr25519::Public::try_from(val)?))
    }
}

impl From<AuraPublic> for Vec<u8> {
    fn from(val: AuraPublic) -> Self {
        val.as_ref().to_vec()
    }
}

impl AsRef<[u8]> for AuraPublic {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}
