//! User management-related API endpoints

use oceaniam_common::{ApiResponse, Empty, RestResult};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{middlewares, state::AppState};

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(users))
}

/// Get user list
#[utoipa::path(
        get,
        path = "/users/",
        tag = "Users",
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
        params(("authorization" = String, Header, description = "Authorization payload"))
    )]
pub async fn users(auth: middlewares::auth::RequireAuth) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}
