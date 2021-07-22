#![cfg(feature = "logic-integration-tests")]

use std::marker::PhantomData;

use facetec_api_client as ft;
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use tokio::sync::{Mutex, MutexGuard};
use tracing::trace;

use crate::{sequence::Sequence, ValidatorPublicKeyToDo};

use super::{Locked, Logic};

struct TestSigner;

#[async_trait::async_trait]
impl super::Signer<Vec<u8>> for TestSigner {
    type Error = ();

    async fn sign<'a, D>(&self, _data: D) -> Result<Vec<u8>, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        Ok(b"dummy signature".to_vec())
    }
}

struct TestParams {
    facetec_test_server_url: String,
    facetec_device_key_identifier: String,
    facetec_injected_ip_address: String,

    enroll_liveness_data: OpaqueLivenessData,
    authenticate_liveness_data: OpaqueLivenessData,
}

impl TestParams {
    pub fn from_env() -> Self {
        let facetec_test_server_url = std::env::var("FACETEC_TEST_SERVER_URL").unwrap();
        let facetec_device_key_identifier = std::env::var("FACETEC_DEVICE_KEY_IDENTIFIER").unwrap();
        let facetec_injected_ip_address = std::env::var("FACETEC_INJECTED_IP_ADDRESS").unwrap();

        let read_liveness_data = |prefix: &str| {
            let read_env_file = |var: &str| {
                let val = std::env::var(format!("{}{}", prefix, var)).unwrap();
                std::fs::read_to_string(val).unwrap()
            };

            let face_scan = read_env_file("FACETEC_FACE_SCAN_PATH");
            let audit_trail_image = read_env_file("FACETEC_AUDIT_TRAIL_IMAGE_PATH");
            let low_quality_audit_trail_image =
                read_env_file("FACETEC_LOW_QUALITY_AUDIT_TRAIL_IMAGE_PATH");

            let liveness_data = LivenessData {
                face_scan,
                audit_trail_image,
                low_quality_audit_trail_image,
            };

            OpaqueLivenessData::from(&liveness_data)
        };

        let enroll_liveness_data = read_liveness_data("ENROLL_");
        let authenticate_liveness_data = read_liveness_data("AUTHENTICATE_");

        Self {
            facetec_test_server_url,
            facetec_device_key_identifier,
            facetec_injected_ip_address,
            enroll_liveness_data,
            authenticate_liveness_data,
        }
    }
}

static LOCK: Mutex<()> = Mutex::const_new(());

async fn setup() -> (
    MutexGuard<'static, ()>,
    TestParams,
    Logic<TestSigner, ValidatorPublicKeyToDo>,
) {
    let guard = LOCK.lock().await;

    let test_params = TestParams::from_env();

    let facetec = ft::Client {
        reqwest: reqwest::Client::new(),
        base_url: test_params.facetec_test_server_url.clone(),
        device_key_identifier: test_params.facetec_device_key_identifier.clone(),
        injected_ip_address: Some(test_params.facetec_injected_ip_address.clone()),
        response_body_error_inspector: crate::LoggingInspector,
    };

    let res = facetec
        .reset()
        .await
        .expect("unable to reset facetec test server");

    trace!(message = "facetec server reset", ?res);

    let locked = Locked {
        sequence: Sequence::new(0),
        execution_id: "test".to_owned(),
        facetec,
        signer: TestSigner,
        public_key_type: PhantomData::<ValidatorPublicKeyToDo>,
    };
    let logic = Logic {
        locked: Mutex::new(locked),
        facetec_device_sdk_params: crate::FacetecDeviceSdkParams {
            device_key_identifier: "device_key_identifier".to_owned(),
            public_face_map_encryption_key: "public_face_map_encryption_key".to_owned(),
        },
    };

    (guard, test_params, logic)
}

#[tokio::test]
#[tracing_test::traced_test]
async fn standalone_enroll() {
    let (_guard, test_params, logic) = setup().await;

    logic
        .enroll(super::op_enroll::Request {
            liveness_data: test_params.enroll_liveness_data,
            public_key: b"dummy validator key".to_vec(),
        })
        .await
        .unwrap();
}

#[tokio::test]
#[tracing_test::traced_test]
async fn first_authenticate() {
    let (_guard, test_params, logic) = setup().await;

    let err = logic
        .authenticate(super::op_authenticate::Request {
            liveness_data: test_params.authenticate_liveness_data,
            liveness_data_signature: b"qwe".to_vec(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, super::op_authenticate::Error::PersonNotFound));
}

#[tokio::test]
#[tracing_test::traced_test]
async fn enroll_authenticate() {
    let (_guard, test_params, logic) = setup().await;

    logic
        .enroll(super::op_enroll::Request {
            liveness_data: test_params.enroll_liveness_data,
            public_key: b"dummy validator key".to_vec(),
        })
        .await
        .unwrap();

    logic
        .authenticate(super::op_authenticate::Request {
            liveness_data: test_params.authenticate_liveness_data,
            liveness_data_signature: b"qwe".to_vec(),
        })
        .await
        .unwrap();
}
