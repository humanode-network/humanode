//! Handlers, the HTTP transport coupling for the internal logic.

use std::sync::Arc;

use serde::Serialize;
use warp::hyper::StatusCode;
use warp::Reply;

use super::error;
use crate::logic::{
    op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
    op_get_public_key, LogicOp,
};

/// Enroll operation HTTP transport coupling.
pub async fn enroll<L>(
    logic: Arc<L>,
    input: op_enroll::Request,
) -> Result<impl warp::Reply, warp::Rejection>
where
    L: LogicOp<op_enroll::Request>,
    L::Error: Into<error::Logic>,
    L::Response: Serialize,
{
    let res = logic.call(input).await.map_err(Into::into)?;

    let reply = warp::reply::json(&res);
    let reply = warp::reply::with_status(reply, StatusCode::CREATED);
    Ok(reply.into_response())
}

/// Authenticate operation HTTP transport coupling.
pub async fn authenticate<L>(
    logic: Arc<L>,
    input: op_authenticate::Request,
) -> Result<impl warp::Reply, warp::Rejection>
where
    L: LogicOp<op_authenticate::Request>,
    L::Error: Into<error::Logic>,
    L::Response: Serialize,
{
    let res = logic.call(input).await.map_err(Into::into)?;

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
    L::Error: Into<error::Logic>,
    L::Response: Serialize,
{
    let res = logic
        .call(op_get_facetec_session_token::Request {})
        .await
        .map_err(Into::into)?;

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
    L::Error: Into<error::Logic>,
    L::Response: Serialize,
{
    let res = logic
        .call(op_get_facetec_device_sdk_params::Request {})
        .await
        .map_err(Into::into)?;

    let reply = warp::reply::json(&res);
    let reply = warp::reply::with_status(reply, StatusCode::OK);
    Ok(reply.into_response())
}

/// Get the robonode public key.
pub async fn get_public_key<L>(logic: Arc<L>) -> Result<impl warp::Reply, warp::Rejection>
where
    L: LogicOp<op_get_public_key::Request>,
    L::Error: Into<error::Logic>,
    L::Response: Serialize,
{
    let res = logic
        .call(op_get_public_key::Request)
        .await
        .map_err(Into::into)?;

    let reply = warp::reply::json(&res);
    let reply = warp::reply::with_status(reply, StatusCode::OK);
    Ok(reply.into_response())
}
