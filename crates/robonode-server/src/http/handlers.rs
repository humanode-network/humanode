//! Handlers, the HTTP transport coupling for the internal logic.

use serde::Serialize;
use std::sync::Arc;
use warp::hyper::StatusCode;
use warp::Reply;

use crate::{
    http::error::ErrorMessage,
    logic::{
        op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
        op_get_public_key, LogicOp,
    },
};

/// This function receives a `Rejection` and tries to return a custom
/// value, otherwise simply passes the rejection along.
pub async fn handle_rejection(
    err: warp::reject::Rejection,
) -> Result<impl Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(op_enroll::Error::InvalidPublicKey) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_PUBLIC_KEY";
    } else if let Some(op_enroll::Error::InvalidLivenessData(_)) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_LIVENESS_DATA_ENROLL";
    } else if let Some(op_enroll::Error::FaceScanRejected) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "FACE_SCAN_REJECTED_ENROLL";
    } else if let Some(op_enroll::Error::PublicKeyAlreadyUsed) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "PUBLIC_KEY_ALREADY_USED";
    } else if let Some(op_enroll::Error::PersonAlreadyEnrolled) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "PERSON_ALREADY_ENROLLED";
    } else if let Some(op_authenticate::Error::InvalidLivenessData(_)) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_LIVENESS_DATA_AUTHENTICATE";
    } else if let Some(op_authenticate::Error::PersonNotFound) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "PERSON_NOT_FOUND";
    } else if let Some(op_authenticate::Error::FaceScanRejected) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "FACE_SCAN_REJECTED_AUTHENTICATE";
    } else if let Some(op_authenticate::Error::SignatureInvalid) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "SIGNATURE_INVALID";
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

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

/// Get the robonode public key.
pub async fn get_public_key<L>(logic: Arc<L>) -> Result<impl warp::Reply, warp::Rejection>
where
    L: LogicOp<op_get_public_key::Request>,
    L::Error: warp::reject::Reject,
    L::Response: Serialize,
{
    let res = logic
        .call(op_get_public_key::Request)
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
impl warp::reject::Reject for op_get_public_key::Error {}
