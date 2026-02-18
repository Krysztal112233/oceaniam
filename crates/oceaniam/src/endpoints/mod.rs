//! API endpoints module
//!
//! Provides HTTP API routes and handlers for the application

use axum::extract::State;
use oceaniam_common::{ApiResponse, Empty, RestResult};
use tap::Pipe as _;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

mod applications;
mod authentication;
mod tenants;
mod users;

#[utoipa::path(
        get,
        path = "/",
        tag = "Meta",
        responses(
            (status = 200, body = ApiResponse<Empty>)
        )
    )]
async fn root(State(_ext): State<AppState>) -> RestResult<Empty> {
    Ok(ApiResponse::default())
}

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(root))
        .pipe(authentication::endpoint)
        .pipe(users::endpoint)
        .pipe(applications::endpoint)
        .pipe(tenants::endpoint)
}
