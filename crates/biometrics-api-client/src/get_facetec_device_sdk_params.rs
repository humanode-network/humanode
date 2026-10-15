//! Client API for the Humanode Biometrics Server.

use reqwest::StatusCode;
use serde_json::{Map, Value};

use crate::{Client, Error};

impl Client {
    /// Perform the facetec-device-sdk-params call to the server.
    pub async fn get_facetec_device_sdk_params(
        &self,
    ) -> Result<GetFacetecDeviceSdkParamsResponse, Error<GetFacetecDeviceSdkParamsError>> {
        let url = format!("{}/facetec-device-sdk-params", self.base_url);
        let res = self.reqwest.get(url).send().await?;
        match res.status() {
            StatusCode::OK => Ok(res.json().await?),
            _ => Err(Error::Call(GetFacetecDeviceSdkParamsError::Unknown(
                res.text().await?,
            ))),
        }
    }
}

/// The parameters necessary to initialize the FaceTec Device SDK.
pub type GetFacetecDeviceSdkParamsResponse = Map<String, Value>;

/// The get-facetec-device-sdk-params-specific error condition.
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GetFacetecDeviceSdkParamsError {
    /// Some error occurred.
    #[error("unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests;
