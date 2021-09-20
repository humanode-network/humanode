//! The validator key integration logic.

use std::convert::{TryFrom, TryInto};

use sp_core::crypto::{CryptoType, Pair, Public};

use crate::logic;

/// A validator public key implemented via substrate crypto.
pub struct SubstratePublic<T: CryptoType>(<T::Pair as Pair>::Public);

#[async_trait::async_trait]
impl<T: CryptoType> logic::Verifier<Vec<u8>> for SubstratePublic<T>
where
    <T::Pair as Pair>::Signature: for<'a> TryFrom<&'a [u8]>,
{
    type Error = ();

    async fn verify<'a, D>(&self, data: D, signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        let data = data.as_ref();
        let signature = signature.as_slice().try_into().map_err(|_| ())?;
        Ok(T::Pair::verify(&signature, &data, &self.0))
    }
}

impl<T: CryptoType> TryFrom<&[u8]> for SubstratePublic<T>
where
    <T::Pair as Pair>::Public: for<'a> TryFrom<&'a [u8]>,
{
    type Error = ();

    fn try_from(val: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(val.try_into().map_err(|_| ())?))
    }
}

impl<T: CryptoType> From<SubstratePublic<T>> for Vec<u8> {
    fn from(val: SubstratePublic<T>) -> Self {
        val.0.to_raw_vec()
    }
}

impl<T: CryptoType> AsRef<[u8]> for SubstratePublic<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}
