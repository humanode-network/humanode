//! Get Facetec Device Sdk Params operation.

use std::convert::TryFrom;

use serde::Serialize;

use super::{Logic, Signer};

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

impl<S, PK> Logic<S, PK>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]>,
{
    /// Get the FaceTec Device SDK params.
    pub async fn get_facetec_device_sdk_params(&self) -> Result<Response, Error> {
        Ok(Response {
            device_key_identifier: self.facetec_device_sdk_params.device_key_identifier.clone(),
            public_face_map_encryption_key: self
                .facetec_device_sdk_params
                .public_face_map_encryption_key
                .clone(),
        })
    }
}
