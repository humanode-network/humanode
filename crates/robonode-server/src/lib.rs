//! Humanode's Bioauth Robonode server internal API.

use std::{convert::Infallible, marker::PhantomData, sync::Arc};

use http::root;
use tokio::sync::Mutex;
use warp::Filter;

mod http;
mod logging_inspector;
mod logic;
mod sequence;
mod validator_key;

pub use logging_inspector::LoggingInspector;
pub use logic::FacetecDeviceSdkParams;

/// Initialize the [`warp::Filter`] implementing the HTTP transport for
/// the robonode.
pub fn init(
    execution_id: uuid::Uuid,
    facetec_api_client: facetec_api_client::Client<LoggingInspector>,
    facetec_device_sdk_params: FacetecDeviceSdkParams,
    robonode_keypair: robonode_crypto::Keypair,
) -> impl Filter<Extract = impl warp::Reply, Error = std::convert::Infallible> + Clone {
    let logic = logic::Logic {
        locked: Mutex::new(logic::Locked {
            sequence: sequence::Sequence::new(0),
            execution_id,
            facetec: facetec_api_client,
            signer: robonode_keypair,
            public_key_type: PhantomData::<validator_key::SubstratePublic<sp_core::sr25519::Public>>,
        }),
        facetec_device_sdk_params,
    };
    let log = warp::log("robonode::api");
    root(Arc::new(logic))
        .with(log)
        .recover(http::rejection::handle)
}

#[async_trait::async_trait]
impl logic::Signer<Vec<u8>> for robonode_crypto::Keypair {
    type Error = Infallible;

    async fn sign<'a, D>(&self, data: D) -> Result<Vec<u8>, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a,
    {
        use robonode_crypto::ed25519_dalek::Signer;
        let sig = Signer::sign(self, data.as_ref());
        Ok(sig.as_ref().to_owned())
    }
}

#[async_trait::async_trait]
impl logic::PublicKeyProvider for robonode_crypto::Keypair {
    fn public_key(&self) -> &[u8] {
        &*self.public.as_bytes()
    }
}
