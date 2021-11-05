//! RPC interface for the bioauth flow.

use std::sync::Arc;

use futures::channel::oneshot;
use futures::lock::BiLock;
use futures::FutureExt;
use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode;
use jsonrpc_derive::rpc;
use primitives_liveness_data::LivenessData;
use serde_json::{Map, Value};

use crate::flow::LivenessDataProvider;

/// A result type that wraps.
pub type Result<T> = std::result::Result<T, RpcError>;

/// A futures that resolves to the specified `T`, or an [`RpcError`].
pub type FutureResult<T> = jsonrpc_core::BoxFuture<Result<T>>;

/// The parameters necessary to initialize the FaceTec Device SDK.
type FacetecDeviceSdkParams = Map<String, Value>;

/// The API exposed via JSON-RPC.
#[rpc]
pub trait BioauthApi {
    /// Get the configuration required for the Device SDK.
    #[rpc(name = "bioauth_getFacetecDeviceSdkParams")]
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams>;

    /// Get a FaceTec Session Token.
    #[rpc(name = "bioauth_getFacetecSessionToken")]
    fn get_facetec_session_token(&self) -> FutureResult<String>;

    /// Provide the liveness data for the currently running enrollemnt or authentication process.
    #[rpc(name = "bioauth_provideLivenessData")]
    fn provide_liveness_data(&self, liveness_data: LivenessData) -> FutureResult<()>;
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
pub struct Bioauth<C>
where
    C: AsRef<robonode_client::Client>,
{
    /// The underlying implementation.
    /// We have to wrap it with `Arc` because `jsonrpc-core` doesn't allow us to use `self` for
    /// the duration of the future; we `clone` the `Arc` to get the `'static` lifetime to
    /// a shared `Inner` instead.
    inner: Arc<Inner<C>>,
}

impl<C> Bioauth<C>
where
    C: AsRef<robonode_client::Client>,
{
    /// Create a new [`Bioauth`] API implementation.
    pub fn new(robonode_client: C, liveness_data_tx_slot: Arc<LivenessDataTxSlot>) -> Self {
        let inner = Inner {
            client: robonode_client,
            liveness_data_tx_slot,
        };
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl<C> Bioauth<C>
where
    C: AsRef<robonode_client::Client> + Send + Sync + 'static,
{
    /// A helper function that provides a conveneient way to to execute a future with a clone of
    /// the `Arc<Inner>`.
    /// It also boxes the resulting [`Future`] `Fut` so it fits into the [`FutureResult`].
    fn with_inner_clone<F, Fut, R>(&self, f: F) -> FutureResult<R>
    where
        F: FnOnce(Arc<Inner<C>>) -> Fut,
        Fut: std::future::Future<Output = Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        let inner = Arc::clone(&self.inner);
        f(inner).boxed()
    }
}

impl<C> BioauthApi for Bioauth<C>
where
    C: AsRef<robonode_client::Client> + Send + Sync + 'static,
{
    /// See [`Inner::get_facetec_device_sdk_params`].
    fn get_facetec_device_sdk_params(&self) -> FutureResult<FacetecDeviceSdkParams> {
        self.with_inner_clone(move |inner| inner.get_facetec_device_sdk_params())
    }

    /// See [`Inner::get_facetec_session_token`].
    fn get_facetec_session_token(&self) -> FutureResult<String> {
        self.with_inner_clone(move |inner| inner.get_facetec_session_token())
    }

    /// See [`Inner::provide_liveness_data`].
    fn provide_liveness_data(&self, liveness_data: LivenessData) -> FutureResult<()> {
        self.with_inner_clone(move |inner| inner.provide_liveness_data(liveness_data))
    }
}

/// The underlying implementation of the RPC part, extracted into a subobject to work around
/// the common pitfall with the poor async engines implementations of requiring future objects to
/// be static.
/// Stop it people, why do you even use Rust if you do things like this? Ffs...
/// See https://github.com/paritytech/jsonrpc/issues/580
struct Inner<C>
where
    C: AsRef<robonode_client::Client>,
{
    /// The robonode client, used for fetching the FaceTec Session Token.
    client: C,
    /// The liveness data provider sink.
    /// We need an [`Arc`] here to allow sharing the data from across multiple invocations of the
    /// RPC extension builder that will be using this RPC.
    liveness_data_tx_slot: Arc<LivenessDataTxSlot>,
}

impl<C> Inner<C>
where
    C: AsRef<robonode_client::Client>,
{
    /// Get the FaceTec Device SDK parameters to use at the device.
    async fn get_facetec_device_sdk_params(self: Arc<Self>) -> Result<FacetecDeviceSdkParams> {
        let res = self
            .client
            .as_ref()
            .get_facetec_device_sdk_params()
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;
        Ok(res)
    }

    /// Get the FaceTec Session Token.
    async fn get_facetec_session_token(self: Arc<Self>) -> Result<String> {
        let res = self
            .client
            .as_ref()
            .get_facetec_session_token()
            .await
            .map_err(|err| RpcError {
                code: ErrorCode::ServerError(1),
                message: format!("request to the robonode failed: {}", err),
                data: None,
            })?;
        Ok(res.session_token)
    }

    /// Collect the liveness data and provide to the consumer.
    async fn provide_liveness_data(self: Arc<Self>, liveness_data: LivenessData) -> Result<()> {
        let maybe_tx = {
            let mut maybe_tx_guard = self.liveness_data_tx_slot.lock().await;
            maybe_tx_guard.take() // take the guarded option value and release the lock asap
        };
        let tx = maybe_tx.ok_or_else(|| RpcError {
            code: ErrorCode::InternalError,
            message: "Flow is not engaged, unable to accept liveness data".into(),
            data: None,
        })?;
        tx.send(liveness_data).map_err(|_| RpcError {
            code: ErrorCode::InternalError,
            message: "Flow was aborted before the liveness data could be submitted".into(),
            data: None,
        })?;
        Ok(())
    }
}

/// Provider implements a [`LivenessDataProvider`].
pub struct Provider {
    /// The shared liveness data sender slot, that we can swap with our ephemernal
    /// channel upon a liveness data reuqest.
    liveness_data_tx_slot: LivenessDataTxSlot,
}

impl Provider {
    /// Construct a new [`Provider`].
    pub fn new(liveness_data_tx_slot: LivenessDataTxSlot) -> Self {
        Self {
            liveness_data_tx_slot,
        }
    }
}

#[async_trait::async_trait]
impl LivenessDataProvider for Provider {
    type Error = oneshot::Canceled;

    async fn provide(&mut self) -> std::result::Result<LivenessData, Self::Error> {
        let (tx, rx) = oneshot::channel();

        {
            let mut maybe_tx_guard = self.liveness_data_tx_slot.lock().await;
            let _ = maybe_tx_guard.insert(tx); // insert a new sender value and free the lock asap
        }

        Ok(rx.await?)
    }
}
