//! Authentication-related API endpoints
//!
//! Provides interfaces for user signin, signup, signout, and token refresh

use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use jsonwebtoken::Header;
use log::error;
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
    State(AppState {
        credentials,
        database,
        mut keybox,
        ..
    }): State<AppState>,

    Json(auth): Json<SystemSigninRequest>,
) -> RestResult<SigninResponse> {
    let (id, password) = match auth {
        SystemSigninRequest::Name { name, password } => (
            Administrators::get_by_name(name, &database).await?.id,
            password,
        ),
    };

    let succeed = {
        let cred = credentials.get_credential(id).await?;
        credential::Password::try_from(&cred)?.verify(password)?
    };

    let Some(keybox) = keybox.get_keybox(consts::SYSTEM_APPLICATION_UUID).await else {
        return Err(Error::with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            "!!!CANNOT FIND SYSTEM KEYBOX, THIS MUST BE ERROR!!!",
        ));
    };

    let Some(key) = keybox.get_latest_raw_key(KeyStatus::Active) else {
        return Err(Error::with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            "!!!CANNOT FIND SYSTEM KEYBOX, THIS MUST BE ERROR!!!",
        ));
    };

    fn h(i: impl JwtCodec<SystemClaim> + 'static) -> Box<dyn JwtCodec<SystemClaim>> {
        Box::new(i)
    }

    let key_alg = key.key_alg.clone();
    let ket = match *key_alg {
        KeyAlg::Ps256
        | KeyAlg::Ps384
        | KeyAlg::Ps512
        | KeyAlg::Rs256
        | KeyAlg::Rs384
        | KeyAlg::Rs512 => h(RsaKey::try_from(key)?),
    };

    let jwt = SystemClaim::new(
        id,
        Duration::from_hours(24 * 5).as_secs() as i64,
        Some(consts::DEFAULT_JWT_ISSUER.into()),
        Some(consts::DEFAULT_JWT_ISSUER.into()),
    )
    .encode(Header::new(key_alg.into()), ket)?;

    Ok(ApiResponse::new(SigninResponse { jwt }))
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
