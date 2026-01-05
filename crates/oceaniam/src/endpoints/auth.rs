use axum::Json;
use oceaniam_common::{ApiResponse, RestResult};
use oceaniam_vo::auth::{SigninResponse, SignoutResponse, SignupRequest, SignupResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(refresh))
        .routes(routes!(signin))
        .routes(routes!(signout))
        .routes(routes!(signup))
}

#[utoipa::path(
        post,
        path = "/auth/signin",
        tag = "Authentication",
        responses(
            (status = 200, body = ApiResponse<SigninResponse>),
        ),
    )]
#[axum::debug_handler]
pub async fn signin() -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

#[utoipa::path(
        get,
        path = "/auth/signout",
        tag = "Authentication",
        responses(
            (status = 200, body = ApiResponse<SignoutResponse>),
        ),
        params(("authorization" = String, Header, description = "Authorization payload")),
    )]
pub async fn signout() -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

#[utoipa::path(
        post,
        path = "/auth/signup",
        tag = "Authentication",
        responses(
            (status = 201, body = ApiResponse<SignupResponse>),
        ),
    )]
pub async fn signup(Json(_request): Json<SignupRequest>) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

#[utoipa::path(
        post,
        path = "/auth/refresh",
        tag = "Authentication",
        responses(
            (status = 200, body = ApiResponse<SigninResponse>),
        ),
        request_body(content_type = "application/json"),
        params(("authorization" = String, Header, description = "Authorization payload")),
    )]
pub async fn refresh() -> RestResult<()> {
    Ok(ApiResponse::new(()))
}
