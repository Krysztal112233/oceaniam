//! Application configuration-related API endpoints

use crate::middlewares::application::AdminJwtOrApplicationSecretGuard;
use crate::{
    endpoints::applications::{TenantApplicationPath, get_tenant_application},
    error::AppResult,
    middlewares::permission::{ApplicationConfigurationPatch, PlatformPermissionGuard},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
};
use oceaniam_api::{ApiResponse, Empty, ErrorResponse};
use oceaniam_audit::types::{AuditPayload, PatchApplicationConfigurationPayload};
use oceaniam_vo::applications::{
    ApplicationConfigurationVO, GetApplicationConfigurationResponse,
    PatchApplicationConfigurationRequest,
};
use tap::Tap;
use tracing::{Span, error, field};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_application_configuration))
        .routes(routes!(patch_application_configuration))
}

/// Get application configuration
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}/configuration",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<GetApplicationConfigurationResponse>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_configuration.get",
    skip(applications, path, database),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn get_application_configuration(
    _: AdminJwtOrApplicationSecretGuard,
    State(AppState {
        applications,
        database,
        ..
    }): State<AppState<'_>>,
    Path(path): Path<TenantApplicationPath>,
) -> AppResult<GetApplicationConfigurationResponse> {
    let application = get_tenant_application(path, &database).await?;
    let application_id = application.id;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application_id));
    });

    let configuration = applications
        .get_configuration(application_id)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to get application configuration"
            )
        })?;

    Ok(ApiResponse::new(GetApplicationConfigurationResponse {
        configuration: ApplicationConfigurationVO::from(configuration),
    }))
}

/// Patch application configuration
#[utoipa::path(
        patch,
        path = "/tenants/{tenant_id}/applications/{application_id}/configuration",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        request_body = PatchApplicationConfigurationRequest,
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_configuration.patch",
    skip(applications, auditing, path, patch, database),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn patch_application_configuration(
    _: PlatformPermissionGuard<ApplicationConfigurationPatch>,

    State(AppState {
        applications,
        auditing,
        database,
        ..
    }): State<AppState<'_>>,
    Path(path): Path<TenantApplicationPath>,
    Json(patch): Json<PatchApplicationConfigurationRequest>,
) -> AppResult<Empty> {
    let application = get_tenant_application(path, &database).await?;
    let application_id = application.id;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application_id));
    });

    applications
        .patch_configuration(application_id, patch)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to patch application configuration"
            )
        })?;

    auditing
        .write(AuditPayload::from(PatchApplicationConfigurationPayload {
            application_id,
        }))
        .await;

    Ok(ApiResponse::new(Empty::default()))
}
