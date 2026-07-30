use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::extract::OptionalQuery;
use oceaniam_api::{ApiResponse, Empty, ErrorResponse, PageParam, PagedResponse};
use oceaniam_audit::types::{
    AuditPayload, CreateApplicationPayload, DeleteApplicationPayload, PatchApplicationPayload,
};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::{
    helper::applications::ApplicationHelper, model, model::prelude::Applications,
};
use oceaniam_vo::applications::{
    ApplicationDetailVO, ApplicationVO, CreateApplicationRequest, CreateApplicationResponse,
    PatchApplicationRequest,
};
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::ResolvedApplication;
use crate::{
    error::AppResult,
    middlewares::application::AdminJwtOrApplicationSecretGuard,
    middlewares::permission::{
        ApplicationCreate, ApplicationDelete, ApplicationPatch, ApplicationRead,
        PlatformPermissionGuard,
    },
    state::AppState,
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(get_applications))
        .routes(routes!(create_application))
        .routes(routes!(get_application))
        .routes(routes!(patch_application))
        .routes(routes!(delete_application))
}

/// Get application list under a tenant
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("page" = Option<u64>, Query, description = "Page number"),
            ("per_page" = Option<u64>, Query, description = "Items per page"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationVO>>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_applications.list",
    skip(database),
    fields(otel.kind = "internal", tenant_id = field::Empty, page = field::Empty, per_page = field::Empty)
)]
pub async fn get_applications(
    _: PlatformPermissionGuard<ApplicationRead>,
    Path(tenant_id): Path<Sqid>,
    OptionalQuery(query): OptionalQuery<PageParam>,
    State(AppState { database, .. }): State<AppState>,
) -> AppResult<PagedResponse<ApplicationVO>> {
    let page = query.unwrap_or_default().into_clamped();
    let tenant_id: Uuid = tenant_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id))
            .record("page", page.page)
            .record("per_page", page.per_page);
    });

    info!(%tenant_id, "getting applications");

    let PagedResponse { items, page_info } =
        Applications::get_applications(tenant_id, Some(page), &database).await?;

    Ok(ApiResponse::new(PagedResponse {
        items: items
            .into_iter()
            .map(crate::conversion::applications::application_model_to_vo)
            .collect(),
        page_info,
    }))
}

/// Create new application under a tenant
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/applications",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
        ),
        request_body = CreateApplicationRequest,
        responses(
            (status = 200, body = ApiResponse<CreateApplicationResponse>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_applications.create",
    skip(applications, auditing, comment),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn create_application(
    _: PlatformPermissionGuard<ApplicationCreate>,
    Path(tenant_id): Path<Sqid>,
    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState>,
    Json(CreateApplicationRequest { comment }): Json<CreateApplicationRequest>,
) -> AppResult<CreateApplicationResponse> {
    let tenant_id: Uuid = tenant_id.try_into()?;
    let model::applications::Model {
        id,
        comment,
        tenant_id,
        ..
    } = applications.create_application(tenant_id, comment).await?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id))
            .record("application_id", field::display(&id));
    });

    info!(
        %tenant_id,
        application_id = %id,
        "application created successfully"
    );

    auditing
        .write(AuditPayload::from(CreateApplicationPayload {
            application_id: id,
            tenant_id,
            comment: comment.clone(),
        }))
        .await;

    Ok(ApiResponse::new(
        crate::conversion::applications::create_application_response(id, tenant_id, comment),
    ))
}

/// Get application detail
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<ApplicationDetailVO>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_applications.get",
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn get_application(
    _: AdminJwtOrApplicationSecretGuard,
    app: ResolvedApplication,
) -> AppResult<ApplicationDetailVO> {
    Ok(ApiResponse::new(
        crate::conversion::applications::application_detail_model_to_vo(app.0),
    ))
}

/// Patch application
#[utoipa::path(
        patch,
        path = "/tenants/{tenant_id}/applications/{application_id}",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        request_body = PatchApplicationRequest,
        responses(
            (status = 200, body = ApiResponse<ApplicationDetailVO>),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_applications.patch",
    skip(applications, auditing, patch),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn patch_application(
    _: PlatformPermissionGuard<ApplicationPatch>,
    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
    Json(patch): Json<PatchApplicationRequest>,
) -> AppResult<ApplicationDetailVO> {
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id));
    });

    let application = applications
        .patch_application(app.id, patch)
        .await
        .inspect_err(
            |e| error!(application_id = %app.id, error = %e, "application patch failed"),
        )?;

    auditing
        .write(AuditPayload::from(PatchApplicationPayload {
            application_id: application.id,
            comment: application.comment.clone(),
        }))
        .await;

    Ok(ApiResponse::new(
        crate::conversion::applications::application_detail_model_to_vo(application),
    ))
}

/// Delete application
#[utoipa::path(
        delete,
        path = "/tenants/{tenant_id}/applications/{application_id}",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_applications.delete",
    skip(applications, auditing),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn delete_application(
    _: PlatformPermissionGuard<ApplicationDelete>,
    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
) -> AppResult<()> {
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id));
    });

    applications.delete_application(app.id).await.inspect_err(
        |e| error!(application_id = %app.id, error = %e, "application deletion failed"),
    )?;

    auditing
        .write(AuditPayload::from(DeleteApplicationPayload {
            application_id: app.id,
        }))
        .await;

    Ok(ApiResponse::new(()))
}
