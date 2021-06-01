//! Humanode's Bioauth Robonode server internal API.

#![warn(missing_docs, clippy::missing_docs_in_private_items)]

use std::{marker::PhantomData, sync::Arc};

use http::root;
use tokio::sync::Mutex;
use warp::Filter;

mod http;
mod logic;
mod sequence;

/// Initialize the [`warp::Filter`] implementing the HTTP transport for
/// the robonode.
pub fn init() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let logic = logic::Logic {
        locked: Mutex::new(logic::Locked {
            sequence: sequence::Sequence::new(0),
            facetec: facetec_api_client::Client {
                base_url: "localhost:5113".to_owned(),
                reqwest: reqwest::Client::new(),
            },
            signer: (),
            public_key_type: PhantomData::<String>,
        }),
    };
    root(Arc::new(logic))
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
