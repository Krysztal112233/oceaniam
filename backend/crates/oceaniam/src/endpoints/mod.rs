//! API endpoints module
//!
//! Provides HTTP API routes and handlers for the application

use crate::error::AppResult;
use axum::extract::State;
use oceaniam_api::{ApiResponse, Empty};
use tap::Pipe as _;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

mod administrators;
pub mod applications;
mod audits;
mod authentication;
mod secrets;
mod statistics;
mod tenants;

#[utoipa::path(
        get,
        path = "/",
        tag = "Meta",
        responses(
            (status = 200, body = ApiResponse<Empty>)
        )
    )]
async fn root(State(_ext): State<AppState>) -> AppResult<Empty> {
    Ok(ApiResponse::default())
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(root))
        .pipe(administrators::endpoint)
        .pipe(applications::endpoint)
        .pipe(authentication::endpoint)
        .pipe(secrets::endpoint)
        .pipe(statistics::endpoint)
        .pipe(audits::endpoint)
        .pipe(tenants::endpoint)
}
