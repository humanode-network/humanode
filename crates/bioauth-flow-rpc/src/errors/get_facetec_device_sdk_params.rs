//! The `get_facetec_device_sdk_params` method error.

use super::{app, ApiErrorCode};

/// The `get_facetec_device_sdk_params` method error kinds.
#[derive(Debug)]
pub enum GetFacetecDeviceSdkParamsError {
    /// An error that can occur during doing a call into robonode.
    Robonode(robonode_client::Error<robonode_client::GetFacetecDeviceSdkParamsError>),
}

impl From<GetFacetecDeviceSdkParamsError> for jsonrpsee::core::Error {
    fn from(err: GetFacetecDeviceSdkParamsError) -> Self {
        match err {
            GetFacetecDeviceSdkParamsError::Robonode(err) => {
                app::simple(ApiErrorCode::Robonode, err.to_string())
            }
        }
    }
}
