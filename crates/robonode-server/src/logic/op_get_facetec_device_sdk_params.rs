//! Get Facetec Device Sdk Params operation.

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use super::{Logic, LogicOp, Signer};

/// The request of the get facetec device sdk params operation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request;

/// The response for the get facetec device sdk params operation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// The public FaceMap encription key.
    pub public_face_map_encryption_key: String,
    /// The device key identifier.
    pub device_key_identifier: String,
}

/// Errors for the get facetec device sdk params operation.
#[derive(Debug)]
pub enum Error {}

#[async_trait::async_trait]
impl<S, PK> LogicOp<Request> for Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]>,
{
    type Response = Response;
    type Error = Error;

    async fn call(&self, _req: Request) -> Result<Self::Response, Self::Error> {
        Ok(Response {
            device_key_identifier: self.facetec_device_sdk_params.device_key_identifier.clone(),
            public_face_map_encryption_key: self
                .facetec_device_sdk_params
                .public_face_map_encryption_key
                .clone(),
        })
    }
}
