use std::sync::Arc;

use facetec_api_client::ServerError;
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

macro_rules! trivial_success_tests {
    (
        $(
            $(#[$test_meta:meta])*
            {
                test_name = $test_name:ident,
                method = $method:expr,
                path = $request:expr,
                input = $input:expr,
                mocked_call = $expect:ident,
                injected_response = $mock_response:ident,
                expected_status = $status_code:expr,
                expected_response = $expected_response:expr,
            },
        )*
    ) => {
        $(
            $(#[$test_meta])*
            #[tokio::test]
            async fn $test_name() {
                let mut mock_logic = MockLogic::new();
                mock_logic.$expect().returning(|_| Ok($mock_response()));

                let filter = root_with_error_handler(mock_logic);

                let res = warp::test::request()
                    .method($method)
                    .path($request)
                    .json(&$input)
                    .reply(&filter)
                    .await;

                // let mut expected_success_response = serde_json::to_string(&$mock_response()).unwrap();

                assert_eq!(res.status(), $status_code);
                assert_eq!(res.body(), &$expected_response);
            }
        )*
    };
}

macro_rules! trivial_error_tests {
    (
        $(
            $(#[$test_meta:meta])*
            {
                test_name = $test_name:ident,
                method = $method:expr,
                path = $request:expr,
                input = $input:expr,
                mocked_call = $expect:ident,
                injected_error = $mock_error:expr,
                expected_status = $status_code:expr,
                expected_code = $error_code:expr,
            },
        )*
    ) => {
        $(
            $(#[$test_meta])*
            #[tokio::test]
            async fn $test_name() {
                let mut mock_logic = MockLogic::new();
                mock_logic.$expect().returning(|_| Err($mock_error));

                let filter = root_with_error_handler(mock_logic);

                let res = warp::test::request()
                    .method($method)
                    .path($request)
                    .json(&$input)
                    .reply(&filter)
                    .await;

                let expected_error_body_response = expect_error_body_response($status_code, $error_code).await;

                assert_eq!(res.status(), $status_code);
                assert_eq!(res.body(), &expected_error_body_response);
            }
        )*
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

fn provide_enroll_response() -> op_enroll::Response {
    op_enroll::Response
}

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

fn provide_public_key() -> op_get_public_key::Response {
    op_get_public_key::Response {
        public_key: b"test_public_key".to_vec(),
    }
}

async fn expect_error_body_response(
    status_code: StatusCode,
    error_code: &'static str,
) -> warp::hyper::body::Bytes {
    let json = warp::reply::json(&rejection::ErrorResponse { error_code });
    let response = warp::reply::with_status(json, status_code).into_response();
    warp::hyper::body::to_bytes(response).await.unwrap()
}

fn root_with_error_handler(
    logic: MockLogic,
) -> impl Filter<Extract = impl warp::Reply, Error = std::convert::Infallible> + Clone {
    root(Arc::new(logic)).recover(rejection::handle)
}

trivial_success_tests! [
    /// This test verifies getting expected HTTP response during succesfull enrollment.
    {
        test_name = enroll_success,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_response = provide_enroll_response,
        expected_status = StatusCode::CREATED,
        expected_response = "",
    },

    /// This test verifies getting expected HTTP response during succesfull authentication request.
    {
        test_name = authenticate_success,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_response = provide_authenticate_response,
        expected_status = StatusCode::OK,
        expected_response = serde_json::to_string(&provide_authenticate_response()).unwrap(),
    },

    /// This test verifies getting expected HTTP response during
    /// succesfull get_facetec_session_token request.
    {
        test_name = get_facetec_session_token_success,
        method = "GET",
        path = "/facetec-session-token",
        input = op_get_facetec_session_token::Request,
        mocked_call = expect_get_facetec_session_token,
        injected_response = provide_facetec_session_token,
        expected_status = StatusCode::OK,
        expected_response = serde_json::to_string(&provide_facetec_session_token()).unwrap(),
    },

    /// This test verifies getting expected HTTP response during
    /// get_facetec_device_sdk_params request in Prod mode.
    {
        test_name = get_facetec_device_sdk_params_in_prod_mode,
        method = "GET",
        path = "/facetec-device-sdk-params",
        input = op_get_facetec_device_sdk_params::Request,
        mocked_call = expect_get_facetec_device_sdk_params,
        injected_response = provide_facetec_device_sdk_params_in_prod_mode,
        expected_status = StatusCode::OK,
        expected_response = serde_json::to_string(&provide_facetec_device_sdk_params_in_prod_mode()).unwrap(),
    },

    /// This test verifies getting expected HTTP response during
    /// get_facetec_device_sdk_params request in Dev mode.
    {
        test_name = get_facetec_device_sdk_params_in_dev_mode,
        method = "GET",
        path = "/facetec-device-sdk-params",
        input = op_get_facetec_device_sdk_params::Request,
        mocked_call = expect_get_facetec_device_sdk_params,
        injected_response = provide_facetec_device_sdk_params_in_dev_mode,
        expected_status = StatusCode::OK,
        expected_response = serde_json::to_string(&provide_facetec_device_sdk_params_in_dev_mode()).unwrap(),
    },

    /// This test verifies getting expected HTTP response during
    /// get_public_key request.
    {
        test_name = get_public_key,
        method = "GET",
        path = "/public-key",
        input = op_get_public_key::Request,
        mocked_call = expect_get_public_key,
        injected_response = provide_public_key,
        expected_status = StatusCode::OK,
        expected_response = serde_json::to_string(&provide_public_key()).unwrap(),
    },
];

trivial_error_tests! [
    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with InvalidPublicKey error.
    {
        test_name = enroll_error_invalid_public_key,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::InvalidPublicKey,
        expected_status = StatusCode::BAD_REQUEST,
        expected_code = "ENROLL_INVALID_PUBLIC_KEY",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with InvalidLivenessData error.
    {
        test_name = enroll_error_invalid_liveness_data,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::InvalidLivenessData(codec::Error::from("invalid_data")),
        expected_status = StatusCode::BAD_REQUEST,
        expected_code = "ENROLL_INVALID_LIVENESS_DATA",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with FaceScanRejected error.
    {
        test_name = enroll_error_face_scan_rejected,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::FaceScanRejected,
        expected_status = StatusCode::FORBIDDEN,
        expected_code = "ENROLL_FACE_SCAN_REJECTED",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with PublicKeyAlreadyUsed error.
    {
        test_name = enroll_error_public_key_already_used,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::PublicKeyAlreadyUsed,
        expected_status = StatusCode::CONFLICT,
        expected_code = "ENROLL_PUBLIC_KEY_ALREADY_USED",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with PersonAlreadyEnrolled error.
    {
        test_name = enroll_error_person_already_enrolled,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::PersonAlreadyEnrolled,
        expected_status = StatusCode::CONFLICT,
        expected_code = "ENROLL_PERSON_ALREADY_ENROLLED",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with InternalErrorEnrollment error.
    {
        test_name = enroll_error_internal_enrollment,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::InternalErrorEnrollment(facetec_api_client::Error::Server(ServerError {
            error_message: "error".to_owned()
        })),
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with InternalErrorEnrollmentUnsuccessful error.
    {
        test_name = enroll_error_internal_enrollment_unsuccessful,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::InternalErrorEnrollmentUnsuccessful,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with InternalErrorDbSearch error.
    {
        test_name = enroll_error_internal_db_search,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::InternalErrorDbSearch(facetec_api_client::Error::Server(ServerError {
            error_message: "error".to_owned()
        })),
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with InternalErrorDbSearchUnsuccessful error.
    {
        test_name = enroll_error_internal_db_search_unsuccessful,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::InternalErrorDbSearchUnsuccessful,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with InternalErrorDbEnroll error.
    {
        test_name = enroll_error_internal_db_enroll,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::InternalErrorDbEnroll(facetec_api_client::Error::Server(ServerError {
            error_message: "error".to_owned()
        })),
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer enrollment request with InternalErrorDbEnrollUnsuccessful error.
    {
        test_name = enroll_error_internal_db_enroll_unsuccessful,
        method = "POST",
        path = "/enroll",
        input = op_enroll::Request {
            public_key: b"key".to_vec(),
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
        },
        mocked_call = expect_enroll,
        injected_error = op_enroll::Error::InternalErrorDbEnrollUnsuccessful,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InvalidLivenessData error.
    {
        test_name = authenticate_error_invalid_liveness_data,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InvalidLivenessData(codec::Error::from("invalid_data")),
        expected_status = StatusCode::BAD_REQUEST,
        expected_code = "AUTHENTICATE_INVALID_LIVENESS_DATA",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with FaceScanRejected error.
    {
        test_name = authenticate_error_face_scan_rejected,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::FaceScanRejected,
        expected_status = StatusCode::FORBIDDEN,
        expected_code = "AUTHENTICATE_FACE_SCAN_REJECTED",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with PersonNotFound error.
    {
        test_name = authenticate_error_person_not_found,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::PersonNotFound,
        expected_status = StatusCode::NOT_FOUND,
        expected_code = "AUTHENTICATE_PERSON_NOT_FOUND",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with SignatureInvalid error.
    {
        test_name = authenticate_error_signature_invalid,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::SignatureInvalid,
        expected_status = StatusCode::FORBIDDEN,
        expected_code = "AUTHENTICATE_SIGNATURE_INVALID",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorEnrollment error.
    {
        test_name = authenticate_error_internall_enrollment,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorEnrollment(facetec_api_client::Error::Server(
            ServerError {
                error_message: "error".to_owned()
            }
        )),
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorEnrollmentUnsuccessful error.
    {
        test_name = authenticate_error_internall_enrollment_unsuccessful,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorEnrollmentUnsuccessful,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorDbSearch error.
    {
        test_name = authenticate_error_internall_db_search,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorDbSearch(facetec_api_client::Error::Server(
            ServerError {
                error_message: "error".to_owned()
            }
        )),
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorDbSearchUnsuccessful error.
    {
        test_name = authenticate_error_internall_db_search_unsuccessful,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorDbSearchUnsuccessful,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorDbSearchMatchLevelMismatch error.
    {
        test_name = authenticate_error_internall_db_search_match_level_mismatch,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorDbSearchMatchLevelMismatch,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorInvalidPublicKeyHex error.
    {
        test_name = authenticate_error_internall_invalid_public_key_hex,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorInvalidPublicKeyHex,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorInvalidPublicKey error.
    {
        test_name = authenticate_error_internall_invalid_public_key,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorInvalidPublicKey,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorSignatureVerificationFailed error.
    {
        test_name = authenticate_error_internall_signature_verification_failed,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorSignatureVerificationFailed,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response
    /// during failer authentication request with InternalErrorAuthTicketSigningFailed error.
    {
        test_name = authenticate_error_internal_auth_ticket_signing_failed,
        method = "POST",
        path = "/authenticate",
        input = op_authenticate::Request {
            liveness_data: OpaqueLivenessData(b"data".to_vec()),
            liveness_data_signature: b"signature".to_vec(),
        },
        mocked_call = expect_authenticate,
        injected_error = op_authenticate::Error::InternalErrorAuthTicketSigningFailed,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },

    /// This test verifies getting expected HTTP response during
    /// failer get_facetec_session_token request with internal error.
    {
        test_name = get_facetec_session_token_error_internal,
        method = "GET",
        path = "/facetec-session-token",
        input = op_get_facetec_session_token::Request,
        mocked_call = expect_get_facetec_session_token,
        injected_error = op_get_facetec_session_token::Error::InternalErrorSessionTokenUnsuccessful,
        expected_status = StatusCode::INTERNAL_SERVER_ERROR,
        expected_code = "LOGIC_INTERNAL_ERROR",
    },
];
