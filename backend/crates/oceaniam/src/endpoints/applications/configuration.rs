use axum::{Json, extract::State};
use oceaniam_api::{ApiResponse, Empty, ErrorResponse};
use oceaniam_audit::types::{AuditPayload, PatchApplicationConfigurationPayload};
use oceaniam_vo::applications::{
    GetApplicationConfigurationResponse, PatchApplicationConfigurationRequest,
};
use tap::Tap;
use tracing::{Span, error, field};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ResolvedApplication;
use crate::{
    error::AppResult,
    middlewares::application::AdminJwtOrApplicationSecretGuard,
    middlewares::permission::{ApplicationConfigurationPatch, PlatformPermissionGuard},
    state::AppState,
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
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
    skip(applications),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn get_application_configuration(
    _: AdminJwtOrApplicationSecretGuard,
    State(AppState { applications, .. }): State<AppState>,
    app: ResolvedApplication,
) -> AppResult<GetApplicationConfigurationResponse> {
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
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
        configuration: crate::conversion::configurations::application_configuration_to_vo(
            configuration,
        ),
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
    skip(applications, auditing, patch),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn patch_application_configuration(
    _: PlatformPermissionGuard<ApplicationConfigurationPatch>,
    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
    Json(patch): Json<PatchApplicationConfigurationRequest>,
) -> AppResult<Empty> {
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
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
