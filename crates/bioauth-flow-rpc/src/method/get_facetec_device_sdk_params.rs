//! The `get_facetec_device_sdk_params` method error.

use crate::error;

/// The `get_facetec_device_sdk_params` method error kinds.
#[derive(Debug)]
pub enum Error {
    /// An error that can occur during doing a call into robonode.
    Robonode(robonode_client::Error<robonode_client::GetFacetecDeviceSdkParamsError>),
}

impl From<Error> for jsonrpsee::core::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Robonode(err) => {
                rpc_error_response::simple(error::code::ROBONODE, err.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use jsonrpsee::types::ErrorObject;

    use super::*;

    #[test]
    fn error_robonode() {
        let error: jsonrpsee::core::Error = Error::Robonode(robonode_client::Error::Call(
            robonode_client::GetFacetecDeviceSdkParamsError::Unknown("test".to_owned()),
        ))
        .into();
        let error: ErrorObject = error.into();

        let expected_error_message =
            "{\"code\":200,\"message\":\"server error: unknown error: test\"}";
        assert_eq!(
            expected_error_message,
            serde_json::to_string(&error).unwrap()
        );
    }
}
