//! Filters, essentially how [`warp`] implements routes and middlewares.

use std::sync::Arc;

use serde::Serialize;
use warp::Filter;

use crate::{
    http::handlers,
    logic::{
        op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
    },
};

use super::traits::LogicOp;

/// Pass the [`Arc`] to the handler.
fn with_arc<T>(
    val: Arc<T>,
) -> impl Filter<Extract = (Arc<T>,), Error = std::convert::Infallible> + Clone
where
    Arc<T>: Send,
{
    warp::any().map(move || Arc::clone(&val))
}

/// Extract the JSON body from the request, rejecting the excessive inputs size.
fn json_body<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
where
    T: Send + for<'de> serde::de::Deserialize<'de>,
{
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 1024 * 16).and(warp::body::json::<T>())
}

/// The root mount point with all the routes.
pub fn root<L>(
    logic: Arc<L>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
    L: LogicOp<op_authenticate::Request>
        + LogicOp<op_enroll::Request>
        + LogicOp<op_get_facetec_device_sdk_params::Request>
        + LogicOp<op_get_facetec_session_token::Request>
        + Send
        + Sync,
    <L as crate::http::traits::LogicOp<crate::logic::op_enroll::Request>>::Error:
        warp::reject::Reject,
    <L as crate::http::traits::LogicOp<crate::logic::op_authenticate::Request>>::Error:
        warp::reject::Reject,
    <L as crate::http::traits::LogicOp<crate::logic::op_authenticate::Request>>::Response:
        Serialize,
    <L as crate::http::traits::LogicOp<crate::logic::op_get_facetec_device_sdk_params::Request>>::Error:
        warp::reject::Reject,
    <L as crate::http::traits::LogicOp<crate::logic::op_get_facetec_device_sdk_params::Request>>::Response:
        Serialize,
    <L as crate::http::traits::LogicOp<crate::logic::op_get_facetec_session_token::Request>>::Error:
        warp::reject::Reject,
    <L as crate::http::traits::LogicOp<crate::logic::op_get_facetec_session_token::Request>>::Response:
        Serialize,
{
    enroll(Arc::clone(&logic))
        .or(authenticate(Arc::clone(&logic)))
        .or(get_facetec_session_token(Arc::clone(&logic)))
        .or(get_facetec_device_sdk_params(logic))
}

/// POST /enroll with JSON body.
fn enroll<L>(
    logic: Arc<L>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
    L: LogicOp<op_enroll::Request> + Send + Sync,
    L::Error: warp::reject::Reject,
{
    warp::path!("enroll")
        .and(warp::post())
        .and(with_arc(logic))
        .and(json_body::<op_enroll::Request>())
        .and_then(handlers::enroll)
}

/// POST /authenticate with JSON body.
fn authenticate<L>(
    logic: Arc<L>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
    L: LogicOp<op_authenticate::Request> + Send + Sync,
    L::Error: warp::reject::Reject,
    L::Response: Serialize,
{
    warp::path!("authenticate")
        .and(warp::post())
        .and(with_arc(logic))
        .and(json_body::<op_authenticate::Request>())
        .and_then(handlers::authenticate)
}

/// GET /facetec-session-token.
fn get_facetec_session_token<L>(
    logic: Arc<L>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
    L: LogicOp<op_get_facetec_session_token::Request> + Send + Sync,
    L::Error: warp::reject::Reject,
    L::Response: Serialize,
{
    warp::path!("facetec-session-token")
        .and(warp::get())
        .and(with_arc(logic))
        .and_then(handlers::get_facetec_session_token)
}

/// GET /facetec-device-sdk-params.
fn get_facetec_device_sdk_params<L>(
    logic: Arc<L>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
    L: LogicOp<op_get_facetec_device_sdk_params::Request> + Send + Sync,
    L::Error: warp::reject::Reject,
    L::Response: Serialize,
{
    warp::path!("facetec-device-sdk-params")
        .and(warp::get())
        .and(with_arc(logic))
        .and_then(handlers::get_facetec_device_sdk_params)
}
