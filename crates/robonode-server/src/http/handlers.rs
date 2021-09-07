//! Handlers, the HTTP transport coupling for the internal logic.

use serde::Serialize;
use std::sync::Arc;
use warp::hyper::StatusCode;
use warp::Reply;

use super::traits::LogicOp;
use crate::logic::{
    op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
};

/// Enroll operation HTTP transport coupling.
pub async fn enroll<L>(
    logic: Arc<L>,
    input: op_enroll::Request,
) -> Result<impl warp::Reply, warp::Rejection>
where
    L: LogicOp<op_enroll::Request>,
    L::Error: warp::reject::Reject,
{
    logic.call(input).await.map_err(warp::reject::custom)?;
    Ok(StatusCode::CREATED)
}

/// Authenticate operation HTTP transport coupling.
pub async fn authenticate<L>(
    logic: Arc<L>,
    input: op_authenticate::Request,
) -> Result<impl warp::Reply, warp::Rejection>
where
    L: LogicOp<op_authenticate::Request>,
    L::Error: warp::reject::Reject,
    L::Response: Serialize,
{
    let res = logic.call(input).await.map_err(warp::reject::custom)?;

    let reply = warp::reply::json(&res);
    let reply = warp::reply::with_status(reply, StatusCode::OK);
    Ok(reply.into_response())
}

/// Get FaceTec Session Token operation HTTP transport coupling.
pub async fn get_facetec_session_token<L>(
    logic: Arc<L>,
) -> Result<impl warp::Reply, warp::Rejection>
where
    L: LogicOp<op_get_facetec_session_token::Request>,
    L::Error: warp::reject::Reject,
    L::Response: Serialize,
{
    let res = logic
        .call(op_get_facetec_session_token::Request {})
        .await
        .map_err(warp::reject::custom)?;

    let reply = warp::reply::json(&res);
    let reply = warp::reply::with_status(reply, StatusCode::OK);
    Ok(reply.into_response())
}

/// Get FaceTec Device SDK Params operation HTTP transport coupling.
pub async fn get_facetec_device_sdk_params<L>(
    logic: Arc<L>,
) -> Result<impl warp::Reply, warp::Rejection>
where
    L: LogicOp<op_get_facetec_device_sdk_params::Request>,
    L::Error: warp::reject::Reject,
    L::Response: Serialize,
{
    let res = logic
        .call(op_get_facetec_device_sdk_params::Request {})
        .await
        .map_err(warp::reject::custom)?;

    let reply = warp::reply::json(&res);
    let reply = warp::reply::with_status(reply, StatusCode::OK);
    Ok(reply.into_response())
}

impl warp::reject::Reject for op_enroll::Error {}
impl warp::reject::Reject for op_authenticate::Error {}
impl warp::reject::Reject for op_get_facetec_device_sdk_params::Error {}
impl warp::reject::Reject for op_get_facetec_session_token::Error {}
