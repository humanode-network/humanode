//! The `get_facetec_device_sdk_params_v2` method error.

use crate::error;

/// The `get_facetec_device_sdk_params_v2` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during doing a call into the biometrics API.
    BiometricsApi(
        biometrics_api_client::Error<biometrics_api_client::GetFacetecDeviceSdkParamsError>,
    ),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::BiometricsApi(err) => {
                rpc_error_response::simple(error::code::BIOMETRICS_API, err.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests;
