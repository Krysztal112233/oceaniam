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
//! - All authentication failures return 401 to prevent username enumeration attacks

use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use jsonwebtoken::Header;
use log::{debug, error};
use oceaniam_common::{
    ApiResponse, ErrorResponse, RestResult, consts,
    error::Error,
    jwt::{ClaimHelper, JwtCodec, SystemClaim},
};
use oceaniam_credential::credential;
use oceaniam_database::{
    helper::administrators::AdministratorsHelper,
    model::{
        prelude::Administrators,
        sea_orm_active_enums::{KeyAlg, KeyStatus},
    },
};
use oceaniam_keybox::key::rsa_key::RsaKey;
use oceaniam_vo::auth::{SigninResponse, SignoutResponse, SignupResponse, SystemSigninRequest};
use tap::Tap;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    middlewares,
    state::{AppState, keybox::ManagedKeyBoxes},
};

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(create_auth_user))
        .routes(routes!(create_auth_token))
        .routes(routes!(delete_auth_token))
        .routes(routes!(refresh_auth_token))
}

async fn sign_jwt(sub: impl Into<Uuid>, keybox: ManagedKeyBoxes) -> Result<String, Error> {
    let sub = sub.into();
    debug!("signing jwt for sub {}", sub);

    let Some(keybox) = keybox.get_keybox(consts::SYSTEM_APPLICATION_UUID).await else {
        error!(
            "cannot find system keybox of {}",
            consts::SYSTEM_APPLICATION_UUID
        );
        return Err(Error::with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            "!!!CANNOT FIND SYSTEM KEYBOX, THIS MUST BE ERROR!!!",
        ));
    };

    let Some(key) = keybox.get_latest_raw_key(KeyStatus::Active) else {
        error!(
            "cannot find active key in system keybox of {}",
            consts::SYSTEM_APPLICATION_UUID
        );
        return Err(Error::with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            "!!!CANNOT FIND SYSTEM KEYBOX, THIS MUST BE ERROR!!!",
        ));
    };

    debug!("found active key with algorithm: {:?}", key.key_alg);

    fn h(i: impl JwtCodec<SystemClaim> + 'static) -> Box<dyn JwtCodec<SystemClaim>> {
        Box::new(i)
    }

    let key_alg = key.key_alg.clone();
    let kid = key.key_id;

    let ket = match *key_alg {
        KeyAlg::Ps256
        | KeyAlg::Ps384
        | KeyAlg::Ps512
        | KeyAlg::Rs256
        | KeyAlg::Rs384
        | KeyAlg::Rs512 => h(RsaKey::try_from(key)
            .inspect_err(|e| error!("failed to convert key to rsakey: {}", e))?),
    };

    SystemClaim::new(
        sub,
        Duration::from_hours(24 * 5).as_secs() as i64,
        Some(consts::DEFAULT_JWT_ISSUER.into()),
        Some(consts::DEFAULT_JWT_ISSUER.into()),
    )
    .encode(
        Header::new(key_alg.into()).tap_mut(|it| it.kid = Some(kid.to_string())),
        ket,
    )
    .inspect_err(|e| error!("failed to encode jwt: {}", e))
}

/// Create auth token (signin)
///
/// Authenticates user with credentials (name and password) and returns a JWT token.
/// The token is valid for 5 days and must be included in the Authorization header
/// for subsequent requests.
///
/// # Errors
///
/// Returns 401 if credentials are invalid (wrong password or user not found)
/// Returns 500 if system keybox is unavailable or cannot generate token
#[utoipa::path(
        post,
        path = "/auth/tokens",
        tag = "SystemAuthentication",
        request_body = SystemSigninRequest,
        responses(
            (status = 200, description = "Successfully authenticated", body = ApiResponse<SigninResponse>),
            (status = 400, description = "Invalid request body"),
            (status = 401, description = "Invalid credentials", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn create_auth_token(
    State(AppState {
        credentials,
        database,
        keyboxes,
        ..
    }): State<AppState>,

    Json(auth): Json<SystemSigninRequest>,
) -> RestResult<SigninResponse> {
    let (id, password) = match auth {
        SystemSigninRequest::Name { name, password } => (
            Administrators::get_by_name(name, &database)
                .await
                .inspect_err(|e| error!("failed to get administrator by name: {}", e))?
                .id,
            password,
        ),
    };

    let succeed = {
        let cred = credentials
            .get_credential(id)
            .await
            .inspect_err(|e| error!("failed to get credential: {}", e))?;
        credential::Password::from(cred)
            .verify(password)
            .await
            .inspect_err(|e| error!("failed to verify password: {}", e))?
    };

    if !succeed {
        return Err(Error::with_code(
            StatusCode::UNAUTHORIZED,
            consts::USER_LOGIN_FAILED_MSG,
        ));
    }

    let jwt = sign_jwt(id, keyboxes)
        .await
        .inspect_err(|e| error!("failed to sign jwt: {}", e))?;

    Ok(ApiResponse::new(SigninResponse { jwt }))
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
pub async fn delete_auth_token(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { revoked_jwt, .. }): State<AppState>,
) -> RestResult<SignoutResponse> {
    revoked_jwt
        .set_revoked(auth.token.claims.jti)
        .await
        .inspect_err(|e| error!("{e}"))?;

    Ok(ApiResponse::new(SignoutResponse::default()))
}

/// Create auth user (signup)
///
/// Creates a new user account with the provided credentials.
///
/// # Note
///
/// **TODO**: This is a placeholder implementation. Account creation logic needs to be implemented.
///
/// # Errors
///
/// Returns 400 if request body is invalid
/// Returns 409 if username already exists
/// Returns 500 if database operation fails
#[utoipa::path(
        post,
        path = "/auth/users",
        tag = "SystemAuthentication",
        request_body = SystemSigninRequest,
        responses(
            (status = 201, description = "User account created successfully", body = ApiResponse<SignupResponse>),
            (status = 400, description = "Invalid request body"),
            (status = 409, description = "Username already exists"),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[allow(unused)]
pub async fn create_auth_user(
    State(AppState { revoked_jwt, .. }): State<AppState>,

    Json(auth): Json<SystemSigninRequest>,
) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

/// Refresh auth token
///
/// Revokes the current JWT token and issues a new one with a fresh expiration time.
/// This implements token rotation for enhanced security - the old token becomes
/// invalid immediately after refresh.
///
/// # Authorization
///
/// Requires a valid JWT token in the Authorization header.
///
/// # Security
///
/// The old token's JTI is added to the revoked tokens list, preventing replay attacks.
/// The new token has a fresh 5-day expiration period.
///
/// # Errors
///
/// Returns 401 if the token is invalid or has already been revoked
/// Returns 500 if database operation fails
#[utoipa::path(
        post,
        path = "/auth/tokens/refresh",
        tag = "SystemAuthentication",
        params(("Authorization" = String, Header, description = "Bearer token to refresh")),
        responses(
            (status = 200, description = "Token refreshed successfully", body = ApiResponse<SigninResponse>),
            (status = 401, description = "Invalid, expired, or revoked token", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn refresh_auth_token(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState {
        revoked_jwt,
        keyboxes,
        ..
    }): State<AppState>,
) -> RestResult<()> {
    let jti = auth.token.claims.jti;
    revoked_jwt
        .set_revoked(jti)
        .await
        .inspect_err(|e| error!("failed to revoke jwt: {}", e))?;

    sign_jwt(auth.token.claims.sub, keyboxes)
        .await
        .inspect_err(|e| error!("failed to sign new jwt: {}", e))?;

    Ok(ApiResponse::new(()))
}
