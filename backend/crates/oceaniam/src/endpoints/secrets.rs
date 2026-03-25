//! Secret management-related API endpoints

use axum::extract::{Path, State};
use oceaniam_audit::types::{
    AuditPayload, CreateApplicationSecretPayload, DeleteApplicationSecretPayload,
};
use oceaniam_common::{
    ApiResponse, Empty, ErrorResponse, PagedResponse, RestResult, types::sqid::Sqid,
};
use oceaniam_vo::applications::SecretVO;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{middlewares::auth::RequireAuth, state::AppState};
use oceaniam_common::jwt::SystemClaim;

#[utoipa::path(
        post,
        path = "/secrets",
        tag = "Secrets",
        params(("Authorization" = String, Header, description = "Bearer token")),
        responses(
            (status = 200, body = ApiResponse<SecretVO>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "secrets.create",
    skip(applications, auditing),
    fields(secret_id = field::Empty)
)]
pub async fn create_secret(
    auth: RequireAuth<SystemClaim>,
    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,
) -> RestResult<SecretVO> {
    let operator_id = auth.token.claims.sub;
    let model = applications
        .secrets()
        .create_secret()
        .await
        .inspect_err(|e| {
            error!(error = %e, "failed to create secret");
        })?;
    Span::current().tap(|it| {
        it.record("secret_id", field::display(&model.id));
    });

    info!(secret_id = %model.id, "secret created successfully");

    auditing
        .write(AuditPayload::from(CreateApplicationSecretPayload {
            operator_id,
            secret_id: model.id,
        }))
        .await;

    Ok(ApiResponse::new(SecretVO::with_unmasked(model)))
}

#[utoipa::path(
        get,
        path = "/secrets",
        tag = "Secrets",
        params(("Authorization" = String, Header, description = "Bearer token")),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<SecretVO>>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(level = "info", name = "secrets.list", skip(applications))]
pub async fn get_secrets(
    _: RequireAuth<SystemClaim>,
    State(AppState { applications, .. }): State<AppState<'_>>,
) -> RestResult<PagedResponse<SecretVO>> {
    let secrets = applications
        .secrets()
        .get_all_secrets()
        .await
        .inspect_err(|e| {
            error!(error = %e, "failed to fetch secrets");
        })?;

    let mut items = Vec::with_capacity(secrets.len());
    for secret in secrets {
        let application_ids = applications
            .secrets()
            .get_secret_application_ids(secret.id)
            .await
            .inspect_err(|e| {
                error!(secret_id = %secret.id, error = %e, "failed to fetch secret bindings");
            })?;
        items.push(SecretVO::with_masked(secret).with_application_ids(application_ids));
    }

    Ok(ApiResponse::new(PagedResponse::with_entire(items)))
}

#[utoipa::path(
        get,
        path = "/secrets/{secret_id}",
        tag = "Secrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("secret_id" = String, Path, description = "Secret ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<SecretVO>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Secret not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "secrets.get",
    skip(applications, secret_id),
    fields(secret_id = field::Empty)
)]
pub async fn get_secret(
    _: RequireAuth<SystemClaim>,
    Path(secret_id): Path<Sqid>,
    State(AppState { applications, .. }): State<AppState<'_>>,
) -> RestResult<SecretVO> {
    let secret_id: Uuid = secret_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert secret_id"))?;
    Span::current().tap(|it| {
        it.record("secret_id", field::display(&secret_id));
    });

    let secret = applications.secrets().get_secret(secret_id).await?;
    let application_ids = applications
        .secrets()
        .get_secret_application_ids(secret_id)
        .await?;

    Ok(ApiResponse::new(
        SecretVO::with_masked(secret).with_application_ids(application_ids),
    ))
}

#[utoipa::path(
        delete,
        path = "/secrets/{secret_id}",
        tag = "Secrets",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("secret_id" = String, Path, description = "Secret ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Secret not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "secrets.delete",
    skip(applications, auditing, secret_id),
    fields(secret_id = field::Empty)
)]
pub async fn delete_secret(
    auth: RequireAuth<SystemClaim>,
    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,
    Path(secret_id): Path<Sqid>,
) -> RestResult<Empty> {
    let operator_id = auth.token.claims.sub;
    let secret_id: Uuid = secret_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert secret_id"))?;
    Span::current().tap(|it| {
        it.record("secret_id", field::display(&secret_id));
    });

    applications
        .secrets()
        .delete_secret_by_id(secret_id)
        .await
        .inspect_err(|e| {
            error!(secret_id = %secret_id, error = %e, "failed to delete secret");
        })?;

    info!(secret_id = %secret_id, "secret deleted successfully");

    auditing
        .write(AuditPayload::from(DeleteApplicationSecretPayload {
            operator_id,
            secret_id,
        }))
        .await;

    Ok(ApiResponse::new(Empty::default()))
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(create_secret))
        .routes(routes!(get_secrets))
        .routes(routes!(get_secret))
        .routes(routes!(delete_secret))
}
