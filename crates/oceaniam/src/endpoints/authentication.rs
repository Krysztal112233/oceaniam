//! Authentication-related API endpoints
//!
//! Provides interfaces for user signin, signup, signout, and token refresh

use axum::{Json, extract::State};
use log::error;
use oceaniam_common::{ApiResponse, ErrorResponse, RestResult, jwt::SystemClaim};
use oceaniam_vo::auth::{SigninResponse, SignoutResponse, SignupResponse, SystemSigninRequest};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{middlewares, state::AppState};

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(refresh))
        .routes(routes!(signin))
        .routes(routes!(signout))
        .routes(routes!(signup))
}

/// User signin
///
/// Authenticates user with credentials
#[utoipa::path(
        post,
        path = "/auth/signin",
        tag = "SystemAuthentication",
        responses(
            (status = 200, body = ApiResponse<SigninResponse>),
            (status = 500, body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn signin(
    State(AppState { revoked_jwt, .. }): State<AppState>,

    Json(auth): Json<SystemSigninRequest>,
) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

/// User signout
///
/// Clears current user session information
#[utoipa::path(
        get,
        path = "/auth/signout",
        tag = "SystemAuthentication",
        params(("Authorization" = String, Header, description = "Authorization payload")),
        responses(
            (status = 200, body = ApiResponse<SignoutResponse>),
            (status = 401, body = ApiResponse<ErrorResponse>),
            (status = 500, body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn signout(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { revoked_jwt, .. }): State<AppState>,
) -> RestResult<SignoutResponse> {
    revoked_jwt
        .set_revoked(auth.token.claims.jti)
        .await
        .inspect_err(|e| error!("{e}"))?;

    Ok(ApiResponse::new(SignoutResponse::default()))
}

/// User signup
///
/// Creates a new user account
#[utoipa::path(
        post,
        path = "/auth/signup",
        tag = "SystemAuthentication",
        responses(
            (status = 201, body = ApiResponse<SignupResponse>),
            (status = 500, body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn signup(
    State(AppState { revoked_jwt, .. }): State<AppState>,

    Json(auth): Json<SystemSigninRequest>,
) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

/// Refresh JWT
///
/// Obtains new access token using refresh token
#[utoipa::path(
        post,
        path = "/auth/refresh",
        tag = "SystemAuthentication",
        params(("Authorization" = String, Header, description = "Authorization payload")),
        responses(
            (status = 200, body = ApiResponse<SigninResponse>),
            (status = 401, body = ApiResponse<ErrorResponse>),
            (status = 500, body = ApiResponse<ErrorResponse>),
        ),
        request_body(content_type = "application/json"),
    )]
pub async fn refresh(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { revoked_jwt, .. }): State<AppState>,
) -> RestResult<()> {
    let jti = auth.token.claims.jti;
    revoked_jwt.set_revoked(jti).await?;

    Ok(ApiResponse::new(()))
}
