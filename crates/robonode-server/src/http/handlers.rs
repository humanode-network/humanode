//! Handlers, the HTTP transport coupling for the internal logic.

use std::{convert::TryFrom, sync::Arc};
use warp::Reply;

use warp::hyper::StatusCode;

use crate::logic::{self, AuthenticateRequest, EnrollRequest, Logic, Signer, Verifier};

/// Enroll operation HTTP transport coupling.
pub async fn enroll<S, PK>(
    logic: Arc<Logic<S, PK>>,
    input: EnrollRequest,
) -> Result<impl warp::Reply, warp::Rejection>
where
    S: Signer + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a str>,
{
    match logic.enroll(input).await {
        Ok(()) => Ok(StatusCode::CREATED),
        Err(err) => Err(warp::reject::custom(err)),
    }
}

/// Authenticate operation HTTP transport coupling.
pub async fn authenticate<S, PK>(
    logic: Arc<Logic<S, PK>>,
    input: AuthenticateRequest,
) -> Result<impl warp::Reply, warp::Rejection>
where
    S: Signer + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a str> + Verifier + Into<Vec<u8>>,
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
    S: Signer + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a str> + Verifier + Into<Vec<u8>>,
{
    match logic.get_facetec_session_token().await {
        Ok(res) => {
            Ok(warp::reply::with_status(warp::reply::json(&res), StatusCode::OK).into_response())
        }
        Err(err) => Err(warp::reject::custom(err)),
    }
}

impl warp::reject::Reject for logic::EnrollError {}
impl warp::reject::Reject for logic::AuthenticateError {}
impl warp::reject::Reject for logic::GetFaceTecSessionTokenError {}
