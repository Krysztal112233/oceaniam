//! Authentication-related API endpoints
//!
//! Provides interfaces for user signin, signup, signout, and token refresh.
//!
//! # Authentication Flow
//!
//! 1. **Signin**: User provides credentials (name/password) and receives a JWT token
//! 2. **Signout**: Current JWT token is revoked and cannot be used anymore
//! 3. **Refresh**: Old JWT token is revoked and a new one is issued
//!
//! # Security Notes
//!
//! - JWT tokens have a limited lifetime (5 days by default)
//! - Revoked tokens are tracked in database and cannot be reused
//! - All authentication failures return 401 to prevent username enumeration

use crate::error::{AppResult, Error};
use axum::{Json, extract::State, http::StatusCode};
use oceaniam_api::{ApiResponse, ErrorResponse};
use oceaniam_audit::types::{AuditPayload, RefreshJwtPayload, RevokeJwtPayload, SignJwtPayload};
use oceaniam_common::consts;
use oceaniam_database::{
    helper::administrators::AdministratorsHelper, model::prelude::Administrators,
};
use oceaniam_vo::auth::{SigninResponseOrChallenge, SignoutResponse, SystemSigninRequest};
use tap::Tap;
use tracing::{Span, error, field};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    middlewares,
    state::{AppState, keybox::EncodedJwt},
    util::{cookie::clear_auth_cookie, token_response::dispatch_signin_response},
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(create_auth_user))
        .routes(routes!(create_auth_token))
        .routes(routes!(delete_auth_token))
        .routes(routes!(refresh_auth_token))
}

/// Create auth token (signin)
///
/// Authenticates user with credentials (name and password) and returns a JWT token.
/// The token is valid for 5 days and must be included in the Authorization header
/// for subsequent requests.
///
/// # Errors
///
/// Returns 401 if credentials are invalid (all failures return 401 to prevent enumeration)
/// Returns 500 if system keybox is unavailable or cannot generate token
#[utoipa::path(
        post,
        path = "/auth/tokens",
        tag = "SystemAuthentication",
        request_body = SystemSigninRequest,
        params(("X-OceanIAM-Token-Dispatch" = Option<String>, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both.")),
        responses(
            (status = 200, description = "Successfully authenticated", body = ApiResponse<SigninResponseOrChallenge>),
            (status = 400, description = "Invalid request body"),
            (status = 401, description = "Invalid credentials", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "auth.signin",
    skip(token_mtd, credentials, database, keyboxes, auditing, auth),
    fields(
        application_id = field::Empty,
        admin_id = field::Empty,
        token_dispatch = field::Empty
    )
)]
pub async fn create_auth_token(
    token_mtd: middlewares::auth::TokenDispatchMethodGuard,

    State(AppState {
        credentials,
        database,
        keyboxes,
        auditing,
        config,
        ..
    }): State<AppState>,

    Json(auth): Json<SystemSigninRequest>,
) -> AppResult<SigninResponseOrChallenge> {
    let (id, password) = match auth {
        SystemSigninRequest::Name { name, password } => (
            Administrators::get_by_name(name, &database)
                .await
                .inspect_err(|e| error!(error = %e, "failed to get administrator by name"))?
                .id,
            password,
        ),
    };
    Span::current().tap(|it| {
        it.record(
            "application_id",
            field::display(&consts::SYSTEM_APPLICATION_UUID),
        )
        .record("admin_id", field::display(&id))
        .record("token_dispatch", field::debug(&token_mtd));
    });

    let succeed = {
        credentials
            .verify_password(id, &password)
            .await
            .inspect_err(|e| error!(admin_id = %id, error = %e, "failed to verify password"))?
    };

    if !succeed {
        return Err(Error::with_code(
            StatusCode::UNAUTHORIZED,
            consts::USER_LOGIN_FAILED_MSG,
        ));
    }

    let EncodedJwt { jwt, claim } = keyboxes
        .sign_system_jwt(id)
        .await
        .inspect_err(|e| error!(admin_id = %id, error = %e, "failed to sign jwt"))?;

    auditing
        .write(AuditPayload::from(SignJwtPayload {
            jti: claim.jti,
            application_id: consts::SYSTEM_APPLICATION_UUID,
            subject_id: id,
        }))
        .await;

    dispatch_signin_response(jwt, &token_mtd, config.cookie.secure)
}

/// Delete auth token (signout)
///
/// Revokes the current JWT token by adding its JTI (JWT ID) to the revoked tokens list.
/// After revocation, the token cannot be used for authentication anymore.
///
/// # Authorization
///
/// Requires a valid JWT token in the Authorization header.
///
/// # Errors
///
/// Returns 401 if the token is invalid or has already been revoked
/// Returns 500 if database operation fails
#[utoipa::path(
        delete,
        path = "/auth/tokens",
        tag = "SystemAuthentication",
        params(("Authorization" = String, Header, description = "Bearer token for authentication")),
        responses(
            (status = 200, description = "Successfully signed out", body = ApiResponse<SignoutResponse>),
            (status = 401, description = "Invalid or expired token"),
            (status = 500, description = "Internal server error"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "auth.signout",
    skip(auth, revoked_jwt, auditing),
    fields(sub = field::Empty, jti = field::Empty)
)]
pub async fn delete_auth_token(
    auth: middlewares::auth::PlatformAuthGuard,
    State(AppState {
        revoked_jwt,
        auditing,
        config,
        ..
    }): State<AppState>,
) -> AppResult<SignoutResponse> {
    Span::current().tap(|it| {
        it.record("sub", field::display(&auth.token.claims.sub))
            .record("jti", field::display(&auth.token.claims.jti));
    });

    revoked_jwt
        .set_revoked(auth.token.claims.jti)
        .await
        .inspect_err(|e| {
            error!(
                sub = %auth.token.claims.sub,
                jti = %auth.token.claims.jti,
                error = %e,
                "failed to revoke jwt"
            )
        })?;

    auditing
        .write(AuditPayload::from(RevokeJwtPayload {
            subject_id: auth.token.claims.sub,
            jti: auth.token.claims.jti,
            application_id: Some(consts::SYSTEM_APPLICATION_UUID),
        }))
        .await;

    let clear = clear_auth_cookie(config.cookie.secure);
    Ok(ApiResponse::new(SignoutResponse::default()).with_cookie(clear)?)
}

/// Create auth user (signup)
///
/// Placeholder endpoint retained for compatibility.
///
/// # Note
///
/// System account creation is not implemented yet. The route remains reserved, but requests
/// return 501 instead of reporting a successful no-op.
///
/// # Errors
///
/// Returns 400 if request body is invalid
/// Returns 501 because system account creation is not implemented
#[utoipa::path(
        post,
        path = "/auth/users",
        tag = "SystemAuthentication",
        request_body = SystemSigninRequest,
        responses(
            (status = 400, description = "Invalid request body"),
            (status = 501, description = "System account creation is not implemented", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(level = "info", name = "auth.signup", skip(_auth))]
pub async fn create_auth_user(Json(_auth): Json<SystemSigninRequest>) -> AppResult<()> {
    Err(Error::with_code(
        StatusCode::NOT_IMPLEMENTED,
        "system account creation is not implemented",
    ))
}

/// Refresh auth token
///
/// Rotates the current JWT by revoking its JTI and issuing a new token.
///
/// The new token can be delivered back to the client via JSON response body and/or
/// an HTTP cookie, controlled by the `X-OceanIAM-Token-Dispatch` request header:
///
/// - `json`: JSON body only
/// - `cookie`: cookie only (JSON body will be empty: `{}`)
/// - `both`: JSON body + cookie (default)
///
/// Cookie name: `auth_token`.
///
/// # Authorization
///
/// Requires `Authorization: Bearer <jwt>`.
///
/// # Security
///
/// The old token's JTI is added to the revoked tokens list, preventing replay attacks.
/// The new token has a fresh 5-day expiration period.
///
/// # Errors
///
/// Returns 203 if the `Authorization` header is missing
/// Returns 400 if the token is malformed, invalid/expired, or already revoked
/// Returns 500 if the database operation fails or a new token cannot be signed
#[utoipa::path(
	        post,
	        path = "/auth/tokens/refresh",
	        tag = "SystemAuthentication",
	        params(
	            ("Authorization" = String, Header, description = "Bearer token to refresh"),
	            ("X-OceanIAM-Token-Dispatch" = Option<String>, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
	        ),
	        responses(
                (status = 200, description = "Token refreshed successfully", body = ApiResponse<SigninResponseOrChallenge>),
	            (status = 203, description = "Missing Authorization header"),
	            (status = 400, description = "Invalid, expired, or revoked token", body = ApiResponse<ErrorResponse>),
	            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
	        ),
	    )]
#[tracing::instrument(
    level = "info",
    name = "auth.refresh",
    skip(auth, token_mtd, revoked_jwt, keyboxes, auditing),
    fields(sub = field::Empty, old_jti = field::Empty, token_dispatch = field::Empty)
)]
pub async fn refresh_auth_token(
    auth: middlewares::auth::PlatformAuthGuard,
    token_mtd: middlewares::auth::TokenDispatchMethodGuard,

    State(AppState {
        revoked_jwt,
        keyboxes,
        auditing,
        config,
        ..
    }): State<AppState>,
) -> AppResult<SigninResponseOrChallenge> {
    let jti = auth.token.claims.jti;
    Span::current().tap(|it| {
        it.record("sub", field::display(&auth.token.claims.sub))
            .record("old_jti", field::display(&jti))
            .record("token_dispatch", field::debug(&token_mtd));
    });

    if revoked_jwt.is_revoked(jti).await? {
        return Err(Error::with_code(
            StatusCode::BAD_REQUEST,
            format!("jwt of jti={jti} has been revoked"),
        ));
    }

    revoked_jwt.set_revoked(jti).await.inspect_err(|e| {
        error!(
            sub = %auth.token.claims.sub,
            %jti,
            error = %e,
            "failed to revoke jwt"
        )
    })?;

    let EncodedJwt { jwt, claim } = keyboxes
        .sign_system_jwt(auth.token.claims.sub)
        .await
        .inspect_err(|e| {
            error!(
                sub = %auth.token.claims.sub,
                old_jti = %jti,
                error = %e,
                "failed to sign refreshed jwt"
            )
        })?;

    auditing
        .write(AuditPayload::from(RefreshJwtPayload {
            application_id: consts::SYSTEM_APPLICATION_UUID,
            subject_id: auth.token.claims.sub,
            old_jti: jti,
            new_jti: claim.jti,
        }))
        .await;

    dispatch_signin_response(jwt, &token_mtd, config.cookie.secure)
}
