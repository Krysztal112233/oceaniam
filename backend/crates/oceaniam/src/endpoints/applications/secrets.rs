use axum::extract::{Path, State};
use oceaniam_api::{ApiResponse, ErrorResponse, PagedResponse};
use oceaniam_vo::applications::SecretVO;
use tap::Tap;
use tracing::{Span, error, field};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::TenantApplicationPath;
use crate::{
    conversion::secrets::{secret_with_masked, with_application_ids},
    error::AppResult,
    middlewares::permission::{PlatformPermissionGuard, SecretRead},
    state::AppState,
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(get_application_secrets))
}

/// List all API secrets bound to an application
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/secrets",
    tag = "ApplicationSecrets",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("tenant_id" = String, Path, description = "Tenant Sqid"),
        ("application_id" = String, Path, description = "Application Sqid"),
    ),
    responses(
        (status = 200, body = ApiResponse<PagedResponse<SecretVO>>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "application_secrets.list",
    skip(applications, path),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn get_application_secrets(
    _: PlatformPermissionGuard<SecretRead>,
    Path(path): Path<TenantApplicationPath>,
    State(AppState { applications, .. }): State<AppState>,
) -> AppResult<PagedResponse<SecretVO>> {
    let tenant_id: Uuid = path.tenant_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert tenant_id");
    })?;
    let application_id: Uuid = path.application_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert application_id");
    })?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id))
            .record("application_id", field::display(&application_id));
    });

    let secrets = applications
        .secrets()
        .get_all_secrets_of(application_id)
        .await
        .inspect_err(|e| {
            error!(%tenant_id, %application_id, error = %e, "failed to fetch application secrets");
        })?;

    let secret_ids: Vec<_> = secrets.iter().map(|s| s.id).collect();
    let application_ids_by_secret = applications
        .secrets()
        .get_secret_application_ids_batch_by_ids(secret_ids)
        .await
        .inspect_err(|e| {
            error!(error = %e, "failed to fetch secret bindings in batch");
        })?;

    let items: Vec<_> = secrets
        .into_iter()
        .map(|secret| {
            let ids = application_ids_by_secret
                .get(&secret.id)
                .cloned()
                .unwrap_or_default();
            with_application_ids(secret_with_masked(secret), ids)
        })
        .collect();

    Ok(ApiResponse::new(PagedResponse::with_entire(items)))
}
