//! API endpoints module
//!
//! Provides HTTP API routes and handlers for the application

use axum::extract::State;
use oceaniam_api::{ApiResponse, Empty, RestResult};
use tap::Pipe as _;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

mod administrators;
mod application_challenges;
mod application_configuration;
mod application_keys;
mod application_tokens;
mod application_users;
mod applications;
mod authentication;
mod secrets;
mod tenants;

#[utoipa::path(
        get,
        path = "/",
        tag = "Meta",
        responses(
            (status = 200, body = ApiResponse<Empty>)
        )
    )]
async fn root(State(_ext): State<AppState<'_>>) -> RestResult<Empty> {
    Ok(ApiResponse::default())
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(root))
        .pipe(administrators::endpoint)
        .pipe(application_configuration::endpoint)
        .pipe(application_challenges::endpoint)
        .pipe(application_keys::endpoint)
        .pipe(application_tokens::endpoint)
        .pipe(application_users::endpoint)
        .pipe(applications::endpoint)
        .pipe(authentication::endpoint)
        .pipe(secrets::endpoint)
        .pipe(tenants::endpoint)
}
