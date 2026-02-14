//! Application management-related API endpoints
//!
//! Provides interfaces for application queries and JWKS retrieval

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use oceaniam_common::{ApiResponse, Empty, ErrorResponse, RestResult, error::Error, jwt::JwkSet};
use oceaniam_vo::applications::CreateApplicationRequest;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::state::AppState;

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
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
    )]
pub async fn get_applications() -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

#[utoipa::path(
        post,
        path = "/applications",
        tag = "Applications",
        params(
            ("application_id", description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
    )]
pub async fn create_application(
    State(AppState {
        mut keybox,
        database,
        ..
    }): State<AppState>,

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
        params(
            ("application_id", description = "Application ID"),
        ),
        responses(
            (status = 200, body = JwkSet),
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
