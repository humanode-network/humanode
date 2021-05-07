//! Humanode's Bioauth Robonode server internal API.

#![deny(missing_docs, clippy::missing_docs_in_private_items)]

use std::sync::Arc;

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
        }),
    };
    root(Arc::new(logic))
}
