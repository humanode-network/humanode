use std::sync::Arc;

use mockall::predicate::*;
use mockall::*;
use primitives_auth_ticket::OpaqueAuthTicket;
use primitives_liveness_data::OpaqueLivenessData;
use sp_application_crypto::sp_core::hexdisplay::AsBytesRef;
use warp::hyper::StatusCode;

use crate::{
    http::root,
    logic::{
        op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
        traits::LogicOp,
    },
};

mock! {
    Logic {
        fn enroll(&self, req: op_enroll::Request) -> Result<op_enroll::Response, op_enroll::Error>;
        fn authenticate(&self, req: op_authenticate::Request) -> Result<op_authenticate::Response, op_authenticate::Error>;
        fn get_facetec_session_token(&self, req: op_get_facetec_session_token::Request) -> Result<op_get_facetec_session_token::Response, op_get_facetec_session_token::Error>;
        fn get_facetec_device_sdk_params(&self, req: op_get_facetec_device_sdk_params::Request) -> Result<op_get_facetec_device_sdk_params::Response, op_get_facetec_device_sdk_params::Error>;
    }
}

macro_rules! impl_LogicOp {
    ($name:ty, $request:ty, $response:ty, $error:ty, $call: ident) => {
        #[async_trait::async_trait]
        impl LogicOp<$request> for $name {
            type Response = $response;
            type Error = $error;

            async fn call(&self, req: $request) -> Result<$response, $error> {
                self.$call(req)
            }
        }
    };
}

impl_LogicOp!(
    MockLogic,
    op_enroll::Request,
    op_enroll::Response,
    op_enroll::Error,
    enroll
);

impl_LogicOp!(
    MockLogic,
    op_authenticate::Request,
    op_authenticate::Response,
    op_authenticate::Error,
    authenticate
);

impl_LogicOp!(
    MockLogic,
    op_get_facetec_session_token::Request,
    op_get_facetec_session_token::Response,
    op_get_facetec_session_token::Error,
    get_facetec_session_token
);

impl_LogicOp!(
    MockLogic,
    op_get_facetec_device_sdk_params::Request,
    op_get_facetec_device_sdk_params::Response,
    op_get_facetec_device_sdk_params::Error,
    get_facetec_device_sdk_params
);

fn provide_authenticate_response() -> op_authenticate::Response {
    op_authenticate::Response {
        auth_ticket: OpaqueAuthTicket(b"ticket".to_vec()),
        auth_ticket_signature: b"signature".to_vec(),
    }
}

fn provide_facetec_session_token() -> op_get_facetec_session_token::Response {
    op_get_facetec_session_token::Response {
        session_token: "token".to_owned(),
    }
}

fn provide_facetec_device_sdk_params() -> op_get_facetec_device_sdk_params::Response {
    op_get_facetec_device_sdk_params::Response {
        public_face_map_encryption_key: "key".to_owned(),
        device_key_identifier: "id".to_owned(),
    }
}

#[tokio::test]
async fn it_works_enroll() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_enroll()
        .returning(|_| Ok(op_enroll::Response));

    let logic = Arc::new(mock_logic);
    let filter = root(logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);
    assert!(res.body().is_empty());
}

#[tokio::test]
async fn it_denies_enroll_with_invalid_public_key() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_enroll()
        .returning(|_| Err(op_enroll::Error::InvalidPublicKey));

    let logic = Arc::new(mock_logic);
    let filter = root(logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(res.body(), "Unhandled rejection: InvalidPublicKey");
}

#[tokio::test]
async fn it_works_authenticate() {
    let input = op_authenticate::Request {
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
        liveness_data_signature: b"signature".to_vec(),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_authenticate()
        .returning(|_| Ok(provide_authenticate_response()));

    let logic = Arc::new(mock_logic);
    let filter = root(logic);

    let res = warp::test::request()
        .method("POST")
        .path("/authenticate")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_response = serde_json::to_string(&provide_authenticate_response()).unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.body().as_bytes_ref(), expected_response.as_bytes());
}

#[tokio::test]
async fn it_denies_authenticate() {
    let input = op_authenticate::Request {
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
        liveness_data_signature: b"signature".to_vec(),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_authenticate()
        .returning(|_| Err(op_authenticate::Error::InternalErrorDbSearchUnsuccessful));

    let logic = Arc::new(mock_logic);
    let filter = root(logic);

    let res = warp::test::request()
        .method("POST")
        .path("/authenticate")
        .json(&input)
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        res.body(),
        "Unhandled rejection: InternalErrorDbSearchUnsuccessful"
    );
}

#[tokio::test]
async fn it_works_get_facetec_session_token() {
    let input = op_get_facetec_session_token::Request;

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_get_facetec_session_token()
        .returning(|_| Ok(provide_facetec_session_token()));

    let logic = Arc::new(mock_logic);
    let filter = root(logic);

    let res = warp::test::request()
        .method("GET")
        .path("/facetec-session-token")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_response = serde_json::to_string(&provide_facetec_session_token()).unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.body().as_bytes_ref(), expected_response.as_bytes());
}

#[tokio::test]
async fn it_denies_get_facetec_session_token() {
    let input = op_get_facetec_session_token::Request;

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_get_facetec_session_token()
        .returning(|_| {
            Err(op_get_facetec_session_token::Error::InternalErrorSessionTokenUnsuccessful)
        });

    let logic = Arc::new(mock_logic);
    let filter = root(logic);

    let res = warp::test::request()
        .method("GET")
        .path("/facetec-session-token")
        .json(&input)
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        res.body(),
        "Unhandled rejection: InternalErrorSessionTokenUnsuccessful"
    );
}

#[tokio::test]
async fn it_works_get_facetec_device_sdk_params() {
    let input = op_get_facetec_device_sdk_params::Request;

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_get_facetec_device_sdk_params()
        .returning(|_| Ok(provide_facetec_device_sdk_params()));

    let logic = Arc::new(mock_logic);
    let filter = root(logic);

    let res = warp::test::request()
        .method("GET")
        .path("/facetec-device-sdk-params")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_response = serde_json::to_string(&provide_facetec_device_sdk_params()).unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.body().as_bytes_ref(), expected_response.as_bytes());
}
