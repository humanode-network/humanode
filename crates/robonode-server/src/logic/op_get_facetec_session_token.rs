//! Get Facetec Session Token operation.

use facetec_api_client as ft;
use serde::{Deserialize, Serialize};

use super::{Logic, LogicOp, Signer};

/// The request of the get facetec session token operation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request;

/// The response for the get facetec session token operation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// The session token returned by the FaceTec Server.
    pub session_token: String,
}

/// Errors for the get facetec session token operation.
///
/// Allow dead code to explicitly control errors data.
#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    /// Internal error at session token retrieval due to the underlying request
    /// error at the API level.
    InternalErrorSessionToken(ft::Error),
    /// Internal error at session token retrieval due to unsuccessful response.
    InternalErrorSessionTokenUnsuccessful,
}

#[async_trait::async_trait]
impl<S, PK> LogicOp<Request> for Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]>,
{
    type Response = Response;
    type Error = Error;

    async fn call(&self, _req: Request) -> Result<Self::Response, Self::Error> {
        let unlocked = self.locked.lock().await;

        let res = unlocked
            .facetec
            .session_token()
            .await
            .map_err(Error::InternalErrorSessionToken)?;

        if !res.success {
            return Err(Error::InternalErrorSessionTokenUnsuccessful);
        }

        Ok(Response {
            session_token: res.session_token,
        })
    }
}
