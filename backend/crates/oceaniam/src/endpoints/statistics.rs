//! Statistics API endpoints

use std::collections::HashMap;

use axum::extract::{Path, State};
use axum_extra::extract::OptionalQuery;
use oceaniam_api::{ApiResponse, ErrorResponse, PageParam, PagedResponse};
use oceaniam_database::helper::statistics::AuditsHelper;
use oceaniam_database::helper::trend;
use oceaniam_database::model::{prelude::Audits, sea_orm_active_enums::AuditType};
use oceaniam_vo::statistics::{
    ApplicationStatisticsVO, ApplicationTrendsVO, AuditLogQuery, AuditLogVO, OverviewVO,
    PlatformTrendsVO, TrendDataPoint, TrendQuery,
};
use tap::Tap;
use tracing::{Span, error, field};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    endpoints::applications::TenantApplicationPath,
    error::AppResult,
    middlewares::permission::{PlatformPermissionGuard, TenantRead},
    state::AppState,
};

use crate::middlewares::application::AdminJwtOrApplicationSecretGuard;

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_statistics))
        .routes(routes!(get_statistics_trends))
        .routes(routes!(get_application_statistics))
        .routes(routes!(get_application_statistics_trends))
        .routes(routes!(get_application_audits))
}

/// Platform overview statistics
#[utoipa::path(
    get,
    path = "/statistics",
    tag = "Statistics",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
    ),
    responses(
        (status = 200, body = ApiResponse<OverviewVO>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(level = "info", name = "statistics", skip(database))]
async fn get_statistics(
    _: PlatformPermissionGuard<TenantRead>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<OverviewVO> {
    let overview: OverviewVO = crate::conversion::statistics::platform_counts_to_overview(
        Audits::platform_counts(&database)
            .await
            .inspect_err(|e| error!(error = %e, "failed to query platform counts"))?,
    );

    Ok(ApiResponse::new(overview))
}

/// Platform-level creation trends over time
#[utoipa::path(
    get,
    path = "/statistics/trends",
    tag = "Statistics",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("granularity" = Option<String>, Query, description = "Aggregation granularity: day, week, or month"),
        ("range" = Option<u64>, Query, description = "Lookback range in days (default 30)"),
    ),
    responses(
        (status = 200, body = ApiResponse<PlatformTrendsVO>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid query parameters", body = ApiResponse<ErrorResponse>),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(level = "info", name = "statistics.trends", skip(database))]
async fn get_statistics_trends(
    _: PlatformPermissionGuard<TenantRead>,
    OptionalQuery(query): OptionalQuery<TrendQuery>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<PlatformTrendsVO> {
    let TrendQuery { granularity, range } = query.unwrap_or_default();

    let granularity_str = granularity.to_string();

    let rows = trend::get_platform_trends(&granularity_str, range, &database)
        .await
        .inspect_err(|e| error!(error = %e, "failed to query platform trends"))?;

    let mut grouped: HashMap<String, Vec<TrendDataPoint>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.entity_type)
            .or_default()
            .push(TrendDataPoint {
                bucket: row.period,
                count: row.count as u64,
            });
    }

    Ok(ApiResponse::new(PlatformTrendsVO {
        granularity,
        range,
        tenants: grouped.remove("tenant").unwrap_or_default(),
        applications: grouped.remove("application").unwrap_or_default(),
        users: grouped.remove("user").unwrap_or_default(),
        administrators: grouped.remove("administrator").unwrap_or_default(),
    }))
}

/// Application-level statistics
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/statistics",
    tag = "Statistics",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("tenant_id" = String, Path, description = "Tenant Sqid"),
        ("application_id" = String, Path, description = "Application Sqid"),
    ),
    responses(
        (status = 200, body = ApiResponse<ApplicationStatisticsVO>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(level = "info", name = "statistics.application", skip(database))]
async fn get_application_statistics(
    _: AdminJwtOrApplicationSecretGuard,

    Path(TenantApplicationPath { application_id, .. }): Path<TenantApplicationPath>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<ApplicationStatisticsVO> {
    let app_id: Uuid = application_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert application_id");
    })?;

    let stats: ApplicationStatisticsVO =
        crate::conversion::statistics::application_counts_to_statistics(
            Audits::application_counts(app_id, &database)
                .await
                .inspect_err(|e| error!(error = %e, "failed to query application counts"))?,
        );

    Ok(ApiResponse::new(stats))
}

/// Application-level creation trends over time
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/statistics/trends",
    tag = "Statistics",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("tenant_id" = String, Path, description = "Tenant Sqid"),
        ("application_id" = String, Path, description = "Application Sqid"),
        ("granularity" = Option<String>, Query, description = "Aggregation granularity: day, week, or month"),
        ("range" = Option<u64>, Query, description = "Lookback range in days (default 30)"),
    ),
    responses(
        (status = 200, body = ApiResponse<ApplicationTrendsVO>),
        (status = 203, description = "Missing Authorization header"),
        (status = 400, description = "Invalid query parameters", body = ApiResponse<ErrorResponse>),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(level = "info", name = "statistics.application.trends", skip(database))]
async fn get_application_statistics_trends(
    _: AdminJwtOrApplicationSecretGuard,

    Path(TenantApplicationPath { application_id, .. }): Path<TenantApplicationPath>,
    OptionalQuery(query): OptionalQuery<TrendQuery>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<ApplicationTrendsVO> {
    let TrendQuery { granularity, range } = query.unwrap_or_default();

    let app_id: Uuid = application_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert application_id");
    })?;

    let granularity_str = granularity.to_string();

    let rows = trend::get_application_trends(app_id, &granularity_str, range, &database)
        .await
        .inspect_err(|e| error!(error = %e, "failed to query application trends"))?;

    let new_users = rows
        .into_iter()
        .map(|r| TrendDataPoint {
            bucket: r.period,
            count: r.count as u64,
        })
        .collect();

    Ok(ApiResponse::new(ApplicationTrendsVO {
        granularity,
        range,
        new_users,
    }))
}

/// Application-scoped audit logs
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/audits",
    tag = "Audits",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("tenant_id" = String, Path, description = "Tenant Sqid"),
        ("application_id" = String, Path, description = "Application Sqid"),
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
    name = "statistics.application.audits",
    skip(database, query),
    fields(page = field::Empty, per_page = field::Empty, audit_type = field::Empty),
)]
async fn get_application_audits(
    _: AdminJwtOrApplicationSecretGuard,

    Path(TenantApplicationPath { application_id, .. }): Path<TenantApplicationPath>,
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

    let app_id: Uuid = application_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert application_id");
    })?;

    let page_param = PageParam { page, per_page };
    let audit_type = audit_type.and_then(|t| t.parse::<AuditType>().ok());

    let PagedResponse { items, page_info } =
        Audits::get_audit_logs_by_app(page_param, app_id, audit_type, &database)
            .await
            .inspect_err(|e| error!(error = %e, "failed to query application audit logs"))?;

    let items = items
        .into_iter()
        .map(crate::conversion::statistics::audit_log_model_to_vo)
        .collect();

    Ok(ApiResponse::new(PagedResponse { items, page_info }))
}
