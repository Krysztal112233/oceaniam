//! Application management-related API endpoints
//!
//! Provides interfaces for application queries and JWKS retrieval

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use oceaniam_common::{
    ApiResponse, Empty, ErrorResponse, PageParam, RestResult,
    error::Error,
    jwks::{JwkSet, JwkSetSchema},
};
use oceaniam_vo::applications::CreateApplicationRequest;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    middlewares::{self},
    state::AppState,
};

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(get_applications))
        .routes(routes!(get_application_jwks))
        .routes(routes!(create_application))
}

/// Get application list
#[utoipa::path(
        get,
        path = "/applications",
        tag = "Applications",
        params(("Authorization" = String, Header, description = "Authorization payload")),
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
    )]
pub async fn get_applications(
    auth: middlewares::auth::RequireAuth,

    State(AppState {
        mut keybox,
        database,
        ..
    }): State<AppState>,
) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

#[utoipa::path(
        post,
        path = "/applications",
        tag = "Applications",
        params(("Authorization" = String, Header, description = "Authorization payload")),
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
    )]
pub async fn create_application(
    auth: middlewares::auth::RequireAuth,

    State(AppState {
        keybox, database, ..
    }): State<AppState>,

    Query(page): Query<Option<PageParam>>,
    Json(CreateApplicationRequest { tenant_id }): Json<CreateApplicationRequest>,
) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

/// Get application JWKS
///
/// Returns the JSON Web Key Set for verifying JWTs issued by this application
#[utoipa::path(
        get,
        path = "/applications/{application_id}/.well-known/jwks.json",
        tag = "Applications",
        responses(
            (status = 200, body = JwkSetSchema),
            (status = 404, body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn get_application_jwks(
    Path(application_id): Path<Uuid>,

    State(AppState { mut keybox, .. }): State<AppState>,
) -> RestResult<JwkSet> {
    let keybox = keybox
        .get_keybox(application_id)
        .await
        .ok_or(Error::with_code(StatusCode::NOT_FOUND, "jwks not found"))?;

    Ok(ApiResponse::new(JwkSet::from(keybox)))
}
