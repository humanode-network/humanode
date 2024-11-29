use std::marker::PhantomData;

use facetec_api_client as ft;
use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use tokio::sync::{Mutex, MutexGuard};
use tracing::{info, trace};

use super::{Locked, Logic, LogicOp};
use crate::{logic::common::DB_GROUP_NAME, sequence::Sequence};

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

struct TestValidatorPublicKey(Vec<u8>);

#[async_trait::async_trait]
impl crate::logic::traits::Verifier<Vec<u8>> for TestValidatorPublicKey {
    type Error = ();

    async fn verify<'a, D>(&self, _data: D, _signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        Ok(true)
    }
}

impl AsRef<[u8]> for TestValidatorPublicKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl<'a> TryFrom<&'a [u8]> for TestValidatorPublicKey {
    type Error = ();

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self(value.to_vec()))
    }
}

impl From<TestValidatorPublicKey> for Vec<u8> {
    fn from(value: TestValidatorPublicKey) -> Self {
        value.0
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

        assert_ne!(enroll_liveness_data, authenticate_liveness_data);

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

const TEST_PUBLIC_KEY: &[u8] = b"dummy validator key";

/// Returns a list of all public keys to cleanup from the FaceTec Server 3D DB.
fn public_keys_to_cleanup() -> Vec<&'static [u8]> {
    vec![TEST_PUBLIC_KEY, b"a", b"b"]
}

async fn setup() -> (
    MutexGuard<'static, ()>,
    TestParams,
    Logic<TestSigner, TestValidatorPublicKey>,
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

    for public_key_to_clenaup in public_keys_to_cleanup() {
        let public_key_hex = hex::encode(public_key_to_clenaup);
        let res = facetec
            .db_delete(ft::db_delete::Request {
                group_name: DB_GROUP_NAME,
                identifier: &public_key_hex,
            })
            .await
            .expect("unable to clear 3D DB at the facetec test server");

        trace!(message = "3D DB cleanup at the facetec server", ?res);
    }

    let locked = Locked {
        sequence: Sequence::new(0),
        execution_id: uuid::Uuid::new_v4(),
        facetec,
        signer: TestSigner,
        public_key_type: PhantomData::<TestValidatorPublicKey>,
    };
    let logic = Logic {
        locked: Mutex::new(locked),
        facetec_device_sdk_params: crate::FacetecDeviceSdkParams {
            device_key_identifier: "device_key_identifier".to_owned(),
            public_face_map_encryption_key: "public_face_map_encryption_key".to_owned(),
            production_key: None,
        },
    };

    (guard, test_params, logic)
}

#[tokio::test]
#[ignore = "requires FaceTec server"]
#[tracing_test::traced_test]
async fn standalone_enroll() {
    let (_guard, test_params, logic) = setup().await;

    logic
        .call(super::op_enroll::Request {
            liveness_data: test_params.enroll_liveness_data,
            liveness_data_signature: b"qwe".to_vec(),
            public_key: TEST_PUBLIC_KEY.to_vec(),
        })
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires FaceTec server"]
#[tracing_test::traced_test]
async fn first_authenticate() {
    let (_guard, test_params, logic) = setup().await;

    let err = logic
        .call(super::op_authenticate::Request {
            liveness_data: test_params.authenticate_liveness_data,
            liveness_data_signature: b"qwe".to_vec(),
        })
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        super::op_authenticate::Error::PersonNotFound(_)
    ));
}

#[tokio::test]
#[ignore = "requires FaceTec server"]
#[tracing_test::traced_test]
async fn enroll_authenticate() {
    let (_guard, test_params, logic) = setup().await;

    logic
        .call(super::op_enroll::Request {
            liveness_data: test_params.enroll_liveness_data,
            liveness_data_signature: b"qwe".to_vec(),
            public_key: TEST_PUBLIC_KEY.to_vec(),
        })
        .await
        .unwrap();

    info!("enroll complete, authenticating now");

    logic
        .call(super::op_authenticate::Request {
            liveness_data: test_params.authenticate_liveness_data,
            liveness_data_signature: b"qwe".to_vec(),
        })
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires FaceTec server"]
#[tracing_test::traced_test]
async fn double_enroll() {
    let (_guard, test_params, logic) = setup().await;

    logic
        .call(super::op_enroll::Request {
            liveness_data: test_params.enroll_liveness_data,
            liveness_data_signature: b"qwe".to_vec(),
            public_key: b"a".to_vec(),
        })
        .await
        .unwrap();

    let err = logic
        .call(super::op_enroll::Request {
            liveness_data: test_params.authenticate_liveness_data,
            liveness_data_signature: b"qwe".to_vec(),
            public_key: b"b".to_vec(),
        })
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        super::op_enroll::Error::PersonAlreadyEnrolled(_)
    ));
}
