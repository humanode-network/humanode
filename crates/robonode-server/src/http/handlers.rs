//! Handlers, the HTTP transport coupling for the internal logic.

use std::{convert::TryFrom, sync::Arc};
use warp::hyper::StatusCode;
use warp::Reply;

use crate::logic::{
    op_authenticate, op_enroll, op_get_facetec_device_sdk_params, op_get_facetec_session_token,
    Logic, Signer, Verifier,
};

/// Enroll operation HTTP transport coupling.
pub async fn enroll<S, PK>(
    logic: Arc<Logic<S, PK>>,
    input: op_enroll::Request,
) -> Result<impl warp::Reply, warp::Rejection>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]> + AsRef<[u8]>,
{
    match logic.enroll(input).await {
        Ok(()) => Ok(StatusCode::CREATED),
        Err(err) => Err(warp::reject::custom(err)),
    }
}

/// Authenticate operation HTTP transport coupling.
pub async fn authenticate<S, PK>(
    logic: Arc<Logic<S, PK>>,
    input: op_authenticate::Request,
) -> Result<impl warp::Reply, warp::Rejection>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + Sync + for<'a> TryFrom<&'a [u8]> + Verifier<Vec<u8>> + Into<Vec<u8>>,
{
    match logic.authenticate(input).await {
        Ok(res) => {
            Ok(warp::reply::with_status(warp::reply::json(&res), StatusCode::OK).into_response())
        }
        Err(err) => Err(warp::reject::custom(err)),
    }
}

/// Get FaceTec Session Token operation HTTP transport coupling.
pub async fn get_facetec_session_token<S, PK>(
    logic: Arc<Logic<S, PK>>,
) -> Result<impl warp::Reply, warp::Rejection>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]>,
{
    match logic.get_facetec_session_token().await {
        Ok(res) => {
            Ok(warp::reply::with_status(warp::reply::json(&res), StatusCode::OK).into_response())
        }
        Err(err) => Err(warp::reject::custom(err)),
    }
}

/// Get FaceTec Device SDK Params operation HTTP transport coupling.
pub async fn get_facetec_device_sdk_params<S, PK>(
    logic: Arc<Logic<S, PK>>,
) -> Result<impl warp::Reply, warp::Rejection>
where
    S: Signer<Vec<u8>> + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a [u8]>,
{
    match logic.get_facetec_device_sdk_params().await {
        Ok(res) => {
            Ok(warp::reply::with_status(warp::reply::json(&res), StatusCode::OK).into_response())
        }
        Err(err) => Err(warp::reject::custom(err)),
    }
}

impl warp::reject::Reject for op_enroll::Error {}
impl warp::reject::Reject for op_authenticate::Error {}
impl warp::reject::Reject for op_get_facetec_device_sdk_params::Error {}
impl warp::reject::Reject for op_get_facetec_session_token::Error {}
