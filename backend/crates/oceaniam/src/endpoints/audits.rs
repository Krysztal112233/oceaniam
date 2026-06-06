//! Audit log API endpoints

use axum::extract::State;
use axum_extra::extract::OptionalQuery;
use oceaniam_api::{ApiResponse, ErrorResponse, PageParam, PagedResponse};
use oceaniam_database::helper::statistics::{AuditLogFinderOpts, AuditStatisticsHelper};
use oceaniam_database::model::{prelude::Audits, sea_orm_active_enums::AuditType};
use oceaniam_vo::statistics::{AuditLogQuery, AuditLogVO};
use tap::Tap;
use tracing::{Span, error, field};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    error::AppResult,
    middlewares::permission::{PlatformPermissionGuard, TenantRead},
    state::AppState,
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router.routes(routes!(get_audit_logs))
}

/// Paginated platform-wide audit logs
#[utoipa::path(
    get,
    path = "/audits",
    tag = "Audits",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("page" = Option<u64>, Query, description = "Page number"),
        ("per_page" = Option<u64>, Query, description = "Items per page"),
        ("audit_type" = Option<String>, Query, description = "Filter by audit type (e.g. SignJwt)"),
    ),
    responses(
        (status = 200, body = ApiResponse<PagedResponse<AuditLogVO>>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "audits.list",
    skip(database, query),
    fields(page = field::Empty, per_page = field::Empty, audit_type = field::Empty),
)]
async fn get_audit_logs(
    _: PlatformPermissionGuard<TenantRead>,
    OptionalQuery(query): OptionalQuery<AuditLogQuery>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<PagedResponse<AuditLogVO>> {
    let AuditLogQuery {
        page,
        per_page,
        audit_type,
    } = query.unwrap_or_default();

    Span::current().tap(|it| {
        it.record("page", page)
            .record("per_page", per_page)
            .record("audit_type", field::debug(&audit_type));
    });

    let page_param = PageParam { page, per_page };
    let audit_type = audit_type.and_then(|t| t.parse::<AuditType>().ok());

    let PagedResponse { items, page_info } =
        Audits::get_audit_logs(
            page_param,
            AuditLogFinderOpts {
                app_id: None,
                audit_type,
            },
            &database,
        )
            .await
            .inspect_err(|e| error!(error = %e, "failed to query audit logs"))?;

    let items = items
        .into_iter()
        .map(crate::conversion::statistics::audit_log_model_to_vo)
        .collect();

    Ok(ApiResponse::new(PagedResponse { items, page_info }))
}
