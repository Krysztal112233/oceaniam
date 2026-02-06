//! Adminstration-related API endpoints
//!
//! Provides interfaces for admin signin and signout
//!
//! NOTE: For security consideration, adminstrator cannot refresh jwt.

use axum::extract::State;
use oceaniam_common::{ApiResponse, Empty, RestResult};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(signin)).routes(routes!(signout))
}

/// Admin signin
///
/// Authenticates admin with credentials
#[utoipa::path(
        get,
        path = "/admin/signin",
        tag = "Adminstration",
        responses(
            (status = 200, body = ApiResponse<Empty>)
        ),
        params(("authorization" = String, Header, description = "Authorization payload")),
    )]
#[axum::debug_handler]
async fn signin(State(_ext): State<AppState>) -> RestResult<Empty> {
    Ok(ApiResponse::default())
}

/// Admin signout
///
/// Clears current admin session information
#[utoipa::path(
        get,
        path = "/admin/signout",
        tag = "Adminstration",
        responses(
            (status = 200, body = ApiResponse<Empty>)
        ),
        params(("authorization" = String, Header, description = "Authorization payload")),
    )]
#[axum::debug_handler]
async fn signout(State(_ext): State<AppState>) -> RestResult<Empty> {
    Ok(ApiResponse::default())
}
