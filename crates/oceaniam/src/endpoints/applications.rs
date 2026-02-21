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
        ApplicationVO, CreateApplicationRequest, CreateApplicationResponse, GetApplicationParam,
    },
    users::UserVO,
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
        tag = "Application",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<UserVO>>),
            (status = 401, description = "Unauthorized"),
            (status = 500, description = "Internal server error"),
        ),
    )]
pub async fn get_application_users(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState>,

    Path(application_id): Path<Sqid>,
    Query(page): Query<PageParam>,
) -> RestResult<PagedResponse<UserVO>> {
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
