use oceaniam_common::{ApiResponse, Empty, RestResult};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(users))
}

#[utoipa::path(
        get,
        path = "/users/",
        tag = "Users",
        responses(
            (status = 200, body = ApiResponse<Empty>),
        )
    )]
#[axum::debug_handler]
pub async fn users() -> RestResult<()> {
    Ok(ApiResponse::new(()))
}
