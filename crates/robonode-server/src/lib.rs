//! Humanode's Bioauth Robonode server internal API.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use std::{convert::Infallible, marker::PhantomData, sync::Arc};

use http::root;
use tokio::sync::Mutex;
use warp::Filter;

mod http;
mod logic;
mod sequence;

pub use logic::FacetecDeviceSdkParams;

/// Initialize the [`warp::Filter`] implementing the HTTP transport for
/// the robonode.
pub fn init(
    facetec_api_client: facetec_api_client::Client,
    facetec_device_sdk_params: FacetecDeviceSdkParams,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let logic = logic::Logic {
        locked: Mutex::new(logic::Locked {
            sequence: sequence::Sequence::new(0),
            facetec: facetec_api_client,
            signer: (),
            public_key_type: PhantomData::<ValidatorPublicKeyToDo>,
        }),
        facetec_device_sdk_params,
    };
    let log = warp::log("robonode::api");
    root(Arc::new(logic)).with(log)
}

// TODO!
impl logic::Signer for () {
    fn sign<D: AsRef<[u8]>>(&self, _data: &D) -> Vec<u8> {
        todo!()
    }
}

/// A temporary validator key mock, that accepts any byte sequences as keys, and consideres any
/// signatures valid.
struct ValidatorPublicKeyToDo(Vec<u8>);

#[async_trait::async_trait]
impl logic::Verifier<Vec<u8>> for ValidatorPublicKeyToDo {
    type Error = Infallible;

    async fn verify<'a, D>(&self, _data: D, _signature: Vec<u8>) -> Result<bool, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        Ok(true)
    }
}

impl std::convert::TryFrom<&str> for ValidatorPublicKeyToDo {
    type Error = ();

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Ok(Self(val.into()))
    }
}

impl From<ValidatorPublicKeyToDo> for Vec<u8> {
    fn from(val: ValidatorPublicKeyToDo) -> Self {
        val.0
    }
}
