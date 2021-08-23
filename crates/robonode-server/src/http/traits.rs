//! The logic-related traits.

use crate::logic::{
    op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
};

/// The trait to make enroll operation.
#[async_trait::async_trait]
pub trait Enroll {
    /// Process Enroll request.
    async fn enroll(&self, req: op_enroll::Request) -> Result<(), op_enroll::Error>;
}

/// The trait to make authenticate operation.
#[async_trait::async_trait]
pub trait Authenticate {
    /// Process Authenticate request.
    async fn authenticate(
        &self,
        req: op_authenticate::Request,
    ) -> Result<op_authenticate::Response, op_authenticate::Error>;
}

/// The trait to make get facetec session token operation.
#[async_trait::async_trait]
pub trait GetFacetecSessionToken {
    /// Process GetFacetecSessionToken request.
    async fn get_facetec_session_token(
        &self,
    ) -> Result<op_get_facetec_session_token::Response, op_get_facetec_session_token::Error>;
}

/// The trait to make get facetec device sdk params operation.
#[async_trait::async_trait]
pub trait GetFacetecDeviceSdkParams {
    /// Process GetFacetecDeviceSdkParams request.
    async fn get_facetec_device_sdk_params(
        &self,
    ) -> Result<op_get_facetec_device_sdk_params::Response, op_get_facetec_device_sdk_params::Error>;
}
