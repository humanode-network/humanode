use crate::logic::{
    op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
};

#[async_trait::async_trait]
pub trait Enroll {
    async fn enroll(&self, req: op_enroll::Request) -> Result<(), op_enroll::Error>;
}

#[async_trait::async_trait]
pub trait Authenticate {
    async fn authenticate(
        &self,
        req: op_authenticate::Request,
    ) -> Result<op_authenticate::Response, op_authenticate::Error>;
}

#[async_trait::async_trait]
pub trait GetFacetecSessionToken {
    async fn get_facetec_session_token(
        &self,
    ) -> Result<op_get_facetec_session_token::Response, op_get_facetec_session_token::Error>;
}

#[async_trait::async_trait]
pub trait GetFacetecDeviceSdkParams {
    async fn get_facetec_device_sdk_params(
        &self,
    ) -> Result<op_get_facetec_device_sdk_params::Response, op_get_facetec_device_sdk_params::Error>;
}
