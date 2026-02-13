//! Application management-related API endpoints
//!
//! Provides interfaces for application queries

use oceaniam_common::{ApiResponse, Empty, RestResult};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(get_applications))
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
