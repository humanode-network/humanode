//! Handlers, the HTTP transport coupling for the internal logic.

use std::{convert::Infallible, sync::Arc};
use warp::Reply;

use warp::hyper::StatusCode;

use crate::logic::{AuthenticateRequest, EnrollRequest, Logic};

/// Enroll operation HTTP transport coupling.
pub async fn enroll(
    logic: Arc<Logic>,
    input: EnrollRequest,
) -> Result<impl warp::Reply, Infallible> {
    match logic.enroll(input).await {
        Ok(()) => Ok(StatusCode::CREATED),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR), // TODO: fix the error handling
    }
}

/// Authenticate operation HTTP transport coupling.
pub async fn authenticate(
    logic: Arc<Logic>,
    input: AuthenticateRequest,
) -> Result<impl warp::Reply, Infallible> {
    match logic.authenticate(input).await {
        Ok(res) => {
            Ok(warp::reply::with_status(warp::reply::json(&res), StatusCode::OK).into_response())
        }
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()), // TODO: fix the error handling
    }
}
