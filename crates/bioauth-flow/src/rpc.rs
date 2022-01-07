//! RPC interface for the bioauth flow.

use std::sync::Arc;

use futures::channel::oneshot;
use futures::lock::BiLock;
use futures::FutureExt;
use jsonrpc_core::Error as RpcError;

use jsonrpc_derive::rpc;
use primitives_liveness_data::LivenessData;
use serde_json::{Map, Value};

use crate::flow::BioauthFlow;

/// A result type that wraps.
pub type Result<T> = std::result::Result<T, RpcError>;

/// A futures that resolves to the specified `T`, or an [`RpcError`].
pub type FutureResult<T> = jsonrpc_core::BoxFuture<Result<T>>;

/// The parameters necessary to initialize the FaceTec Device SDK.
pub type FacetecDeviceSdkParams = Map<String, Value>;

/// The API exposed via JSON-RPC.
#[rpc]
pub trait BioauthApi {
    /// Get the configuration required for the Device SDK.
    #[rpc(name = "bioauth_getFacetecDeviceSdkParams")]
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams>;

    /// Get a FaceTec Session Token.
    #[rpc(name = "bioauth_getFacetecSessionToken")]
    fn get_facetec_session_token(&self) -> FutureResult<String>;

    /// Enroll with provided liveness data.
    #[rpc(name = "bioauth_enroll")]
    fn enroll(&self, liveness_data: LivenessData) -> FutureResult<()>;

    /// Authenticate with provided liveness data.
    #[rpc(name = "bioauth_authenticate")]
    fn authenticate(&self, liveness_data: LivenessData) -> FutureResult<()>;
}

/// The shared [`LivenessData`] sender slot, that we can swap with our ephemernal
/// channel upon a liveness data request.
pub type LivenessDataTxSlot = BiLock<Option<oneshot::Sender<LivenessData>>>;

/// Create an linked pair of an empty [`LivenessDataTxSlot`]s.
/// To be used in the initialization process.
pub fn new_liveness_data_tx_slot() -> (LivenessDataTxSlot, LivenessDataTxSlot) {
    BiLock::new(None)
}

/// The RPC implementation.
pub struct Bioauth<B>
where
    B: BioauthFlow,
{
    /// The underlying implementation.
    /// We have to wrap it with `Arc` because `jsonrpc-core` doesn't allow us to use `self` for
    /// the duration of the future; we `clone` the `Arc` to get the `'static` lifetime to
    /// a shared `Inner` instead.
    flow: Arc<B>,
}

impl<B> Bioauth<B>
where
    B: BioauthFlow,
{
    /// Create a new [`Bioauth`] API implementation.
    pub fn new(handler: B) -> Self {
        Self {
            flow: handler.into(),
        }
    }
}

impl<B> Bioauth<B>
where
    B: BioauthFlow + Send + Sync + 'static,
{
    /// A helper function that provides a convenient way to execute a future with a clone of
    /// the `Arc<Inner>`.
    /// It also boxes the resulting [`Future`] `Fut` so it fits into the [`FutureResult`].
    fn with_flow_clone<F, Fut, R>(&self, f: F) -> FutureResult<R>
    where
        F: FnOnce(Arc<B>) -> Fut,
        Fut: std::future::Future<Output = Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        let handler = Arc::clone(&self.flow);
        f(handler).boxed()
    }
}

impl<B> BioauthApi for Bioauth<B>
where
    B: BioauthFlow + Send + Sync + 'static,
{
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams> {
        self.with_flow_clone(
            move |flow| async move { Ok(flow.get_facetec_device_sdk_params().await?) },
        )
    }

    fn get_facetec_session_token(&self) -> FutureResult<String> {
        self.with_flow_clone(move |flow| async move { Ok(flow.get_facetec_session_token().await?) })
    }

    fn enroll(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_flow_clone(move |flow| async move { Ok(flow.enroll(liveness_data).await?) })
    }

    fn authenticate(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_flow_clone(move |flow| async move { Ok(flow.authenticate(liveness_data).await?) })
    }
}
