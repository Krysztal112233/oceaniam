//! Application management-related API endpoints

use crate::error::AppResult;
use crate::error::Error;
use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::extract::OptionalQuery;
use oceaniam_api::{ApiResponse, Empty, ErrorResponse, PageParam, PagedResponse};
use oceaniam_audit::types::{
    AuditPayload, CreateApplicationPayload, DeleteApplicationPayload, PatchApplicationPayload,
};
use oceaniam_auth::{
    jwks::{JwkSet, JwkSetSchema},
    jwt::SystemClaim,
};
use oceaniam_database::{
    helper::applications::ApplicationHelper, model, model::prelude::Applications,
};
use oceaniam_vo::applications::{
    ApplicationDetailVO, ApplicationVO, CreateApplicationRequest, CreateApplicationResponse,
    PatchApplicationRequest,
};
use oceaniam_vo::sqid::Sqid;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{middlewares, state::AppState};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(create_application))
        .routes(routes!(get_application))
        .routes(routes!(patch_application))
        .routes(routes!(delete_application))
        .routes(routes!(get_application_jwks))
        .routes(routes!(get_applications))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub(crate) struct TenantApplicationPath {
    pub tenant_id: Sqid,
    pub application_id: Sqid,
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
    fields(tenant_id = field::Empty, page = field::Empty, per_page = field::Empty)
)]
pub async fn get_applications(
    _: middlewares::auth::RequireAuth<SystemClaim>,
    Path(tenant_id): Path<Sqid>,
    OptionalQuery(query): OptionalQuery<PageParam>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<PagedResponse<ApplicationVO>> {
    let page = query.unwrap_or_default();
    let tenant_id: Uuid = tenant_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id))
            .record("page", page.page)
            .record("per_page", page.per_page);
    });

    info!(tenant_id = %tenant_id, "getting applications");

    let PagedResponse { items, page_info } =
        Applications::get_applications(tenant_id, page, &database).await?;

    Ok(ApiResponse::new(PagedResponse {
        items: items.into_iter().map(ApplicationVO::from).collect(),
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
    skip(applications, auditing, keyboxes, comment),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn create_application(
    _: middlewares::auth::RequireAuth<SystemClaim>,
    Path(tenant_id): Path<Sqid>,
    State(AppState {
        applications,
        auditing,
        keyboxes,
        ..
    }): State<AppState<'_>>,
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
        tenant_id = %tenant_id,
        application_id = %id,
        "application created successfully"
    );

    keyboxes.create_keybox(id).await?;

    info!(
        tenant_id = %tenant_id,
        application_id = %id,
        "default keybox of application created successfully"
    );

    auditing
        .write(AuditPayload::from(CreateApplicationPayload {
            application_id: id,
            tenant_id,
            comment: comment.clone(),
        }))
        .await;

    Ok(ApiResponse::new(CreateApplicationResponse {
        tenant_id: tenant_id.into(),
        application_id: id.into(),
        comment,
    }))
}

/// Get application detail
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<ApplicationDetailVO>),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_applications.get",
    skip(database, path),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn get_application(
    _: middlewares::auth::RequireAuth<SystemClaim>,
    Path(path): Path<TenantApplicationPath>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<ApplicationDetailVO> {
    let application = get_tenant_application(path, &database).await?;

    Ok(ApiResponse::new(ApplicationDetailVO::from(application)))
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
    skip(applications, auditing, path, patch),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn patch_application(
    _: middlewares::auth::RequireAuth<SystemClaim>,
    Path(path): Path<TenantApplicationPath>,
    State(AppState {
        database,
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,
    Json(patch): Json<PatchApplicationRequest>,
) -> AppResult<ApplicationDetailVO> {
    let application = get_tenant_application(path, &database).await?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application.id));
    });

    let application = applications
        .patch_application(application.id, patch)
        .await
        .inspect_err(
            |e| error!(application_id = %application.id, error = %e, "application patch failed"),
        )?;

    auditing
        .write(AuditPayload::from(PatchApplicationPayload {
            application_id: application.id,
            comment: application.comment.clone(),
        }))
        .await;

    Ok(ApiResponse::new(ApplicationDetailVO::from(application)))
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
    skip(applications, auditing, path),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn delete_application(
    _: middlewares::auth::RequireAuth<SystemClaim>,

    Path(path): Path<TenantApplicationPath>,
    State(AppState {
        database,
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,
) -> AppResult<()> {
    let application = get_tenant_application(path, &database).await?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application.id));
    });

    applications
        .delete_application(application.id)
        .await
        .inspect_err(
            |e| error!(application_id = %application.id, error = %e, "application deletion failed"),
        )?;

    auditing
        .write(AuditPayload::from(DeleteApplicationPayload {
            application_id: application.id,
        }))
        .await;

    Ok(ApiResponse::new(()))
}

/// Get application JWKS
#[utoipa::path(
        get,
        path = "/applications/{application_id}/.well-known/jwks.json",
        tag = "Applications",
        responses(
            (status = 200, body = JwkSetSchema),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.jwks",
    skip(keyboxes, application_id),
    fields(application_id = field::Empty)
)]
pub async fn get_application_jwks(
    Path(application_id): Path<Sqid>,
    State(AppState { keyboxes, .. }): State<AppState<'_>>,
) -> AppResult<JwkSet> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id));
    });

    Ok(ApiResponse::new(
        keyboxes
            .get_jwks(application_id)
            .await
            .inspect_err(|e| error!(%application_id, error = %e, "failed to get jwks"))?,
    ))
}

pub(crate) async fn get_tenant_application(
    TenantApplicationPath {
        tenant_id,
        application_id,
    }: TenantApplicationPath,
    database: &sea_orm::DatabaseConnection,
) -> Result<model::applications::Model, Error> {
    let tenant_id: Uuid = tenant_id.try_into()?;
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id))
            .record("application_id", field::display(&application_id));
    });

    let application =
        oceaniam_database::model::prelude::Applications::get_application(application_id, database)
            .await
            .inspect_err(|e| {
                error!(
                    %tenant_id,
                    %application_id,
                    error = %e,
                    "failed to get application detail"
                )
            })?;

    if application.tenant_id != tenant_id {
        return Err(Error::with_code(
            axum::http::StatusCode::NOT_FOUND,
            format!(
                "application_id={} not found under tenant_id={}",
                application.id, tenant_id
            ),
        ));
    }

    Ok(application)
}
