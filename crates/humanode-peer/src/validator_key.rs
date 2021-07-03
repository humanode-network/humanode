//! The validator key integration logic.

use std::convert::Infallible;

use bioauth_flow::flow::Signer;

/// A temporary fake implementation of the validator key, for the purposes of using it with the
/// bioauth enroll and authenticate during the integration while the real validator key is not
/// ready.
pub struct FakeTodo(pub &'static str);

#[async_trait::async_trait]
impl Signer<Vec<u8>> for FakeTodo {
    type Error = Infallible;

    async fn sign<'a, D>(&self, _data: D) -> Result<Vec<u8>, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        Ok(b"0123456789abcdef0123456789abcdef"[..].into())
    }
}

impl AsRef<[u8]> for FakeTodo {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
