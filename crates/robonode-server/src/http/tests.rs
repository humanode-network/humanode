use std::sync::Arc;

use mockall::predicate::*;
use mockall::*;
use primitives_auth_ticket::OpaqueAuthTicket;
use primitives_liveness_data::OpaqueLivenessData;
use warp::{hyper::StatusCode, Filter, Reply};

use crate::{
    http::{rejection, root},
    logic::{
        op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
        op_get_public_key, LogicOp,
    },
};

mock! {
    Logic {
        fn enroll(&self, req: op_enroll::Request) -> Result<op_enroll::Response, op_enroll::Error>;
        fn authenticate(&self, req: op_authenticate::Request) -> Result<op_authenticate::Response, op_authenticate::Error>;
        fn get_facetec_session_token(&self, req: op_get_facetec_session_token::Request) -> Result<op_get_facetec_session_token::Response, op_get_facetec_session_token::Error>;
        fn get_facetec_device_sdk_params(&self, req: op_get_facetec_device_sdk_params::Request) -> Result<op_get_facetec_device_sdk_params::Response, op_get_facetec_device_sdk_params::Error>;
        fn get_public_key(&self, req: op_get_public_key::Request) -> Result<op_get_public_key::Response, op_get_public_key::Error>;
    }
}

macro_rules! impl_LogicOp {
    ($name:ty, $request:ty, $response:ty, $error:ty, $call: ident) => {
        #[async_trait::async_trait]
        impl LogicOp<$request> for $name {
            type Response = $response;
            type Error = $error;

            async fn call(&self, req: $request) -> Result<Self::Response, Self::Error> {
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

impl_LogicOp!(
    MockLogic,
    op_get_public_key::Request,
    op_get_public_key::Response,
    op_get_public_key::Error,
    get_public_key
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

fn provide_facetec_device_sdk_params_in_dev_mode() -> op_get_facetec_device_sdk_params::Response {
    op_get_facetec_device_sdk_params::Response {
        public_face_map_encryption_key: "key".to_owned(),
        device_key_identifier: "id".to_owned(),
        production_key: None,
    }
}

fn provide_facetec_device_sdk_params_in_prod_mode() -> op_get_facetec_device_sdk_params::Response {
    op_get_facetec_device_sdk_params::Response {
        public_face_map_encryption_key: "key".to_owned(),
        device_key_identifier: "id".to_owned(),
        production_key: Some("ProdKey".to_owned()),
    }
}

fn root_with_error_handler(
    logic: MockLogic,
) -> impl Filter<Extract = impl warp::Reply, Error = std::convert::Infallible> + Clone {
    root(Arc::new(logic)).recover(rejection::handle)
}

async fn expect_body_response(
    status_code: StatusCode,
    error_code: &'static str,
) -> warp::hyper::body::Bytes {
    let json = warp::reply::json(&rejection::ErrorResponse { error_code });
    let response = warp::reply::with_status(json, status_code).into_response();
    warp::hyper::body::to_bytes(response).await.unwrap()
}

/// This test verifies getting expected HTTP response during succesfull enrollment.
#[tokio::test]
async fn enroll_success() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_enroll()
        .returning(|_| Ok(op_enroll::Response));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);
    assert!(res.body().is_empty());
}

/// This test verifies getting expected HTTP response
/// during failer enrollment request with InvalidPublicKey error.
#[tokio::test]
async fn enroll_error_invalid_public_key() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_enroll()
        .returning(|_| Err(op_enroll::Error::InvalidPublicKey));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::BAD_REQUEST, "ENROLL_INVALID_PUBLIC_KEY").await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer enrollment request with InvalidLivenessData error.
#[tokio::test]
async fn enroll_error_invalid_liveness_data() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic.expect_enroll().returning(|_| {
        Err(op_enroll::Error::InvalidLivenessData(codec::Error::from(
            "invalid_data",
        )))
    });

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::BAD_REQUEST, "ENROLL_INVALID_LIVENESS_DATA").await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer enrollment request with FaceScanRejected error.
#[tokio::test]
async fn enroll_error_face_scan_rejected() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_enroll()
        .returning(|_| Err(op_enroll::Error::FaceScanRejected));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::FORBIDDEN, "ENROLL_FACE_SCAN_REJECTED").await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer enrollment request with PublicKeyAlreadyUsed error.
#[tokio::test]
async fn enroll_error_public_key_already_used() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_enroll()
        .returning(|_| Err(op_enroll::Error::PublicKeyAlreadyUsed));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::CONFLICT, "ENROLL_PUBLIC_KEY_ALREADY_USED").await;

    assert_eq!(res.status(), StatusCode::CONFLICT);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer enrollment request with PersonAlreadyEnrolled error.
#[tokio::test]
async fn enroll_error_person_already_enrolled() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_enroll()
        .returning(|_| Err(op_enroll::Error::PersonAlreadyEnrolled));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::CONFLICT, "ENROLL_PERSON_ALREADY_ENROLLED").await;

    assert_eq!(res.status(), StatusCode::CONFLICT);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer enrollment request with internal error.
#[tokio::test]
async fn enroll_error_internal() {
    let input = op_enroll::Request {
        public_key: b"key".to_vec(),
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_enroll()
        .returning(|_| Err(op_enroll::Error::InternalErrorEnrollmentUnsuccessful));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/enroll")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::INTERNAL_SERVER_ERROR, "LOGIC_INTERNAL_ERROR").await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response during succesfull authentication request.
#[tokio::test]
async fn authenticate_success() {
    let input = op_authenticate::Request {
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
        liveness_data_signature: b"signature".to_vec(),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_authenticate()
        .returning(|_| Ok(provide_authenticate_response()));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/authenticate")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_response = serde_json::to_string(&provide_authenticate_response()).unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.body().as_ref(), expected_response.as_bytes());
}

/// This test verifies getting expected HTTP response
/// during failer authentication request with InvalidLivenessData error.
#[tokio::test]
async fn authenticate_error_invalid_liveness_data() {
    let input = op_authenticate::Request {
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
        liveness_data_signature: b"signature".to_vec(),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic.expect_authenticate().returning(|_| {
        Err(op_authenticate::Error::InvalidLivenessData(
            codec::Error::from("invalid_data"),
        ))
    });

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/authenticate")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response = expect_body_response(
        StatusCode::BAD_REQUEST,
        "AUTHENTICATE_INVALID_LIVENESS_DATA",
    )
    .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer authentication request with FaceScanRejected error.
#[tokio::test]
async fn authenticate_error_face_scan_rejected() {
    let input = op_authenticate::Request {
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
        liveness_data_signature: b"signature".to_vec(),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_authenticate()
        .returning(|_| Err(op_authenticate::Error::FaceScanRejected));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/authenticate")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::FORBIDDEN, "AUTHENTICATE_FACE_SCAN_REJECTED").await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer authentication request with PersonNotFound error.
#[tokio::test]
async fn authenticate_error_person_not_found() {
    let input = op_authenticate::Request {
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
        liveness_data_signature: b"signature".to_vec(),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_authenticate()
        .returning(|_| Err(op_authenticate::Error::PersonNotFound));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/authenticate")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::NOT_FOUND, "AUTHENTICATE_PERSON_NOT_FOUND").await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer authentication request with SignatureInvalid error.
#[tokio::test]
async fn authenticate_error_signature_invalid() {
    let input = op_authenticate::Request {
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
        liveness_data_signature: b"signature".to_vec(),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_authenticate()
        .returning(|_| Err(op_authenticate::Error::SignatureInvalid));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/authenticate")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::FORBIDDEN, "AUTHENTICATE_SIGNATURE_INVALID").await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response
/// during failer authentication request with internal error.
#[tokio::test]
async fn authenticate_error_internal() {
    let input = op_authenticate::Request {
        liveness_data: OpaqueLivenessData(b"data".to_vec()),
        liveness_data_signature: b"signature".to_vec(),
    };

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_authenticate()
        .returning(|_| Err(op_authenticate::Error::InternalErrorEnrollmentUnsuccessful));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("POST")
        .path("/authenticate")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::INTERNAL_SERVER_ERROR, "LOGIC_INTERNAL_ERROR").await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response during
/// succesfull get_facetec_session_token request.
#[tokio::test]
async fn get_facetec_session_token_success() {
    let input = op_get_facetec_session_token::Request;

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_get_facetec_session_token()
        .returning(|_| Ok(provide_facetec_session_token()));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("GET")
        .path("/facetec-session-token")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_response = serde_json::to_string(&provide_facetec_session_token()).unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.body().as_ref(), expected_response.as_bytes());
}

/// This test verifies getting expected HTTP response during
/// failer get_facetec_session_token request with internal error.
#[tokio::test]
async fn get_facetec_session_token_error_internal() {
    let input = op_get_facetec_session_token::Request;

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_get_facetec_session_token()
        .returning(|_| {
            Err(op_get_facetec_session_token::Error::InternalErrorSessionTokenUnsuccessful)
        });

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("GET")
        .path("/facetec-session-token")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_body_response =
        expect_body_response(StatusCode::INTERNAL_SERVER_ERROR, "LOGIC_INTERNAL_ERROR").await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(res.body(), &expected_body_response);
}

/// This test verifies getting expected HTTP response during
/// get_facetec_device_sdk_params request in Dev mode.
#[tokio::test]
async fn get_facetec_device_sdk_params_in_dev_mode() {
    let input = op_get_facetec_device_sdk_params::Request;

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_get_facetec_device_sdk_params()
        .returning(|_| Ok(provide_facetec_device_sdk_params_in_dev_mode()));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("GET")
        .path("/facetec-device-sdk-params")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_response =
        serde_json::to_string(&provide_facetec_device_sdk_params_in_dev_mode()).unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.body().as_ref(), expected_response.as_bytes());
}

/// This test verifies getting expected HTTP response during
/// get_facetec_device_sdk_params request in Prod mode.
#[tokio::test]
async fn get_facetec_device_sdk_params_in_prod_mode() {
    let input = op_get_facetec_device_sdk_params::Request;

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_get_facetec_device_sdk_params()
        .returning(|_| Ok(provide_facetec_device_sdk_params_in_prod_mode()));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("GET")
        .path("/facetec-device-sdk-params")
        .json(&input)
        .reply(&filter)
        .await;

    let expected_response =
        serde_json::to_string(&provide_facetec_device_sdk_params_in_prod_mode()).unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.body().as_ref(), expected_response.as_bytes());
}

/// This test verifies getting expected HTTP response during
/// get_public_key request.
#[tokio::test]
async fn get_public_key() {
    let input = op_get_public_key::Request;

    let sample_response = op_get_public_key::Response {
        public_key: b"test_public_key".to_vec(),
    };
    let expected_response = serde_json::to_string(&sample_response).unwrap();

    let mut mock_logic = MockLogic::new();
    mock_logic
        .expect_get_public_key()
        .once()
        .returning(move |_| Ok(sample_response.clone()));

    let filter = root_with_error_handler(mock_logic);

    let res = warp::test::request()
        .method("GET")
        .path("/public-key")
        .json(&input)
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.body().as_ref(), expected_response.as_bytes());
}
