//! Get robonode public key operation.

use serde::{Deserialize, Serialize};

use super::{Logic, LogicOp, PublicKeyProvider};

/// The request of the get robonode public key  operation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request;

/// The response for the get robonode public key  operation.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// The public key of the robonode.
    pub public_key: Vec<u8>,
}

/// Errors for the get robonode public key operation.
#[derive(Debug)]
pub enum Error {}

#[async_trait::async_trait]
impl<S, PK> LogicOp<Request> for Logic<S, PK>
where
    S: PublicKeyProvider + Send + 'static,
    PK: Send,
{
    type Response = Response;
    type Error = Error;

    async fn call(&self, _req: Request) -> Result<Self::Response, Self::Error> {
        let unlocked = self.locked.lock().await;
        let public_key = unlocked.signer.public_key().to_vec();
        Ok(Response { public_key })
    }
}
