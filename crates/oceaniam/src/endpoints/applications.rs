//! Application management-related API endpoints
//!
//! Provides interfaces for application queries and JWKS retrieval

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use log::{error, info};
use oceaniam_common::{
    ApiResponse, Empty, ErrorResponse, PageParam, PagedResponse, RestResult, consts,
    error::Error,
    jwks::{JwkSet, JwkSetSchema},
    jwt::SystemClaim,
    types::sqid::Sqid,
};
use oceaniam_database::{
    helper::{
        applications::{ApplicationHelper, CreateApplicationOptions},
        users::UserHelper,
    },
    model::{self, prelude::*, sea_orm_active_enums::KeyAlg},
};
use oceaniam_keybox::{KeyBox, key::rsa_key::RsaKey, keybox::KeyOption};
use oceaniam_vo::{
    applications::{
        ApplicationUserVO, ApplicationVO, CreateApplicationRequest, CreateApplicationResponse,
        GetApplicationParam,
    },
    auth::{SigninResponse, SignoutResponse},
};
use sea_orm::TransactionTrait;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    middlewares::{self},
    state::AppState,
};

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(create_application))
        .routes(routes!(delete_application))
        .routes(routes!(get_application_jwks))
        .routes(routes!(get_applications))
        .routes(routes!(get_application_users))
        .routes(routes!(create_application_user))
        .routes(routes!(create_application_auth_token))
        .routes(routes!(delete_application_auth_token))
        .routes(routes!(refresh_application_auth_token))
        .routes(routes!(create_application_secret))
        .routes(routes!(get_application_secrets))
        .routes(routes!(get_application_secret))
        .routes(routes!(delete_application_secret))
}

/// Get application list
#[utoipa::path(
        get,
        path = "/applications",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Authorization payload"),
            GetApplicationParam
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationVO>>),
            (status = 401, body = ApiResponse<ErrorResponse>),
            (status = 500, body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn get_applications(
    _: middlewares::auth::RequireAuth<SystemClaim>,
    Query(GetApplicationParam {
        tenant_id,
        page,
        per_page,
    }): Query<GetApplicationParam>,
    State(AppState { database, .. }): State<AppState>,
) -> RestResult<PagedResponse<ApplicationVO>> {
    let PagedResponse { items, page_info } = Applications::get_applications(
        tenant_id.try_into()?,
        PageParam { page, per_page },
        &database,
    )
    .await?;

    Ok(ApiResponse::new(PagedResponse {
        items: items.into_iter().map(ApplicationVO::from).collect(),
        page_info,
    }))
}

/// Create new application
///
/// Creates a new application with an automatically generated RSA key pair
#[utoipa::path(
        post,
        path = "/applications",
        tag = "Applications",
        params(("Authorization" = String, Header, description = "Authorization payload")),
        responses(
            (status = 200, body = ApiResponse<CreateApplicationResponse>),
            (status = 401, body = ApiResponse<ErrorResponse>),
            (status = 500, body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn create_application(
    _: middlewares::auth::RequireAuth<SystemClaim>,

    State(AppState { database, .. }): State<AppState>,

    Json(CreateApplicationRequest { tenant_id, comment }): Json<CreateApplicationRequest>,
) -> RestResult<CreateApplicationResponse> {
    let database = database.begin().await?;

    let model::applications::Model {
        id,
        comment,
        tenant_id,
    } = Applications::create_with_opts(
        Uuid::now_v7(),
        tenant_id.clone().try_into()?,
        CreateApplicationOptions { comment },
        &database,
    )
    .await
    .inspect_err(|e| error!("{e}"))?;

    let mut keybox = KeyBox::new(id);
    keybox
        .put_key_with_option(
            RsaKey::new(Uuid::now_v7(), KeyAlg::Ps512),
            KeyOption {
                retired_at: Some((Utc::now() + consts::DEFAULT_KEY_RETIED_AFTER).into()),
                expires_at: Some((Utc::now() + consts::DEFAULT_KEY_EXPIRES_AFTER).into()),
                ..Default::default()
            },
        )
        .inspect_err(|e| error!("{e}"))?;
    keybox
        .write_to(&database)
        .await
        .inspect_err(|e| error!("{e}"))?;
    database
        .commit()
        .await
        .inspect_err(|e| error!("{e}"))
        .inspect(|_| info!("application created successfully: id={}", id))?;

    Ok(ApiResponse::new(CreateApplicationResponse {
        tenant_id: tenant_id.into(),
        application_id: Sqid::from(id),
        comment,
    }))
}

/// Delete application
///
/// Permanently removes an application and all associated data
#[utoipa::path(
        delete,
        path = "/applications",
        tag = "Applications",
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 401, body = ApiResponse<ErrorResponse>),
            (status = 404, body = ApiResponse<ErrorResponse>),
            (status = 500, body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn delete_application(
    Path(application_id): Path<Sqid>,

    State(AppState { database, .. }): State<AppState>,
) -> RestResult<()> {
    let application_id = Uuid::try_from(application_id)?;

    Applications::delete_application(application_id, &database)
        .await
        .inspect_err(|e| error!("{e}"))
        .inspect(|_| info!("application deleted successfully: id={}", application_id))?;

    Ok(ApiResponse::new(()))
}

/// Get application JWKS
///
/// Returns the JSON Web Key Set for verifying JWTs issued by this application
#[utoipa::path(
        get,
        path = "/applications/{application_id}/.well-known/jwks.json",
        tag = "Applications",
        responses(
            (status = 200, body = JwkSetSchema),
            (status = 401, body = ApiResponse<ErrorResponse>),
            (status = 404, body = ApiResponse<ErrorResponse>),
            (status = 500, body = ApiResponse<ErrorResponse>),
        ),
    )]
pub async fn get_application_jwks(
    Path(application_id): Path<Sqid>,

    State(AppState { keybox, .. }): State<AppState>,
) -> RestResult<JwkSet> {
    let application_id = Uuid::try_from(application_id)?;

    Ok(ApiResponse::new(
        keybox
            .get_jwks(application_id)
            .await
            .ok_or(Error::with_code(StatusCode::NOT_FOUND, "jwks not found"))?,
    ))
}

/// Get user list
#[utoipa::path(
        get,
        path = "/applications/{application_id}/users/",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationUserVO>>),
            (status = 401, description = "Unauthorized"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn get_application_users(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,

    Path(application_id): Path<Sqid>,
    Query(page): Query<PageParam>,
) -> RestResult<PagedResponse<ApplicationUserVO>> {
    let operator_id = auth.token.claims.sub;

    let PagedResponse { items, page_info } =
        Users::get_users(application_id.try_into()?, page, &database)
            .await
            .inspect_err(|e| {
                error!("user list query failed: operator_id={operator_id}, error={e}",)
            })?;

    Ok(ApiResponse::new(PagedResponse {
        items: items.into_iter().map(Into::into).collect(),
        page_info,
    }))
}

/// Create application user (signup)
#[utoipa::path(
        post,
        path = "/applications/{application_id}/users",
        tag = "ApplicationUsers",
        params(
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 201, body = ApiResponse<ApplicationUserVO>),
            (status = 400, description = "Bad request"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn create_application_user(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,

    Path(application_id): Path<Sqid>,
) -> RestResult<ApplicationUserVO> {
    todo!()
}

/// Create auth token (signin)
#[utoipa::path(
        post,
        path = "/applications/{application_id}/auth/tokens",
        tag = "ApplicationUserAuthentication",
        params(
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<SigninResponse>),
            (status = 400, description = "Invalid credentials"),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn create_application_auth_token(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,

    Path(application_id): Path<Sqid>,
) -> RestResult<SigninResponse> {
    todo!()
}

/// Delete auth token (signout)
#[utoipa::path(
        delete,
        path = "/applications/{application_id}/auth/tokens",
        tag = "ApplicationUserAuthentication",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<SignoutResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn delete_application_auth_token(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,

    Path(application_id): Path<Sqid>,
) -> RestResult<SignoutResponse> {
    todo!()
}

/// Refresh auth token
#[utoipa::path(
        post,
        path = "/applications/{application_id}/auth/tokens/refresh",
        tag = "ApplicationUserAuthentication",
        params(
            ("Authorization" = String, Header, description = "Bearer refresh token"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<SigninResponse>),
            (status = 401, description = "Invalid or expired refresh token"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn refresh_application_auth_token(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,

    Path(application_id): Path<Sqid>,
) -> RestResult<SigninResponse> {
    todo!()
}

/// Create application secret
///
/// Creates a new API secret for the specified application
#[utoipa::path(
        post,
        path = "/applications/{application_id}/secrets",
        tag = "ApplicationSecrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn create_application_secret(
    _auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,
    Path(application_id): Path<Sqid>,
) -> RestResult<()> {
    let _application_id = Uuid::try_from(application_id)?;
    todo!()
}

/// Get application secrets
///
/// Returns a paginated list of API secrets for the specified application
#[utoipa::path(
        get,
        path = "/applications/{application_id}/secrets",
        tag = "ApplicationSecrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn get_application_secrets(
    _auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,
    Path(application_id): Path<Sqid>,
) -> RestResult<()> {
    let _application_id = Uuid::try_from(application_id)?;
    todo!()
}

/// Get application secret
///
/// Returns detailed information about a specific API secret
#[utoipa::path(
        get,
        path = "/applications/{application_id}/secrets/{secret_id}",
        tag = "ApplicationSecrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
            ("secret_id" = String, Path, description = "Secret ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Secret not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn get_application_secret(
    _auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,
    Path((application_id, secret_id)): Path<(Sqid, Sqid)>,
) -> RestResult<()> {
    let _application_id = Uuid::try_from(application_id)?;
    let _secret_id = Uuid::try_from(secret_id)?;
    todo!()
}

/// Delete application secret
///
/// Revokes (soft deletes) the specified API secret for an application
#[utoipa::path(
        delete,
        path = "/applications/{application_id}/secrets/{secret_id}",
        tag = "ApplicationSecrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
            ("secret_id" = String, Path, description = "Secret ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Secret not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn delete_application_secret(
    _auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,
    Path((application_id, secret_id)): Path<(Sqid, Sqid)>,
) -> RestResult<()> {
    let _application_id = Uuid::try_from(application_id)?;
    let _secret_id = Uuid::try_from(secret_id)?;
    todo!()
}
