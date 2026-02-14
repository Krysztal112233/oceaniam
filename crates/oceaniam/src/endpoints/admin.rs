//! Adminstration-related API endpoints
//!
//! Provides interfaces for admin signin and signout
//!
//! NOTE: For security consideration, adminstrator cannot refresh jwt.

use axum::extract::State;
use oceaniam_common::{ApiResponse, Empty, RestResult};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{middlewares, state::AppState};

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
        params(("Authorization" = String, Header, description = "Authorization payload")),
    )]

async fn signin(
    auth: middlewares::auth::RequireAuth,
    State(_ext): State<AppState>,
) -> RestResult<Empty> {
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
        params(("Authorization" = String, Header, description = "Authorization payload")),
    )]
async fn signout(
    auth: middlewares::auth::RequireAuth,
    State(_ext): State<AppState>,
) -> RestResult<Empty> {
    Ok(ApiResponse::default())
}
