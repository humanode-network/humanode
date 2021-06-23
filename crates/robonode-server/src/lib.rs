//! Humanode's Bioauth Robonode server internal API.

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::clone_on_ref_ptr
)]

use std::{marker::PhantomData, sync::Arc};

use http::root;
use tokio::sync::Mutex;
use warp::Filter;

mod http;
mod logic;
mod sequence;

pub use logic::FaceTecDeviceSdkParams;

/// Initialize the [`warp::Filter`] implementing the HTTP transport for
/// the robonode.
pub fn init(
    facetec_api_client: facetec_api_client::Client,
    facetec_device_sdk_params: FaceTecDeviceSdkParams,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let logic = logic::Logic {
        locked: Mutex::new(logic::Locked {
            sequence: sequence::Sequence::new(0),
            facetec: facetec_api_client,
            signer: (),
            public_key_type: PhantomData::<String>,
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

// TODO!
impl logic::Verifier for String {
    fn verify<D: AsRef<[u8]>, S: AsRef<[u8]>>(&self, _data: &D, _signature: &S) -> bool {
        todo!()
    }
}
