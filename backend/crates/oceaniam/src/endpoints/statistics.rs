//! Statistics API endpoints — platform-level handlers only.
//! Application-scoped statistics handlers live in `applications/statistics.rs`.

use std::collections::HashMap;

use axum::extract::State;
use axum_extra::extract::OptionalQuery;
use oceaniam_api::{ApiResponse, ErrorResponse};
use oceaniam_database::helper::statistics::AuditsHelper;
use oceaniam_database::helper::trend;
use oceaniam_database::model::prelude::Audits;
use oceaniam_vo::statistics::{OverviewVO, PlatformTrendsVO, TrendDataPoint, TrendQuery};
use tracing::error;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    error::AppResult,
    middlewares::permission::{PlatformPermissionGuard, TenantRead},
    state::AppState,
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_statistics))
        .routes(routes!(get_statistics_trends))
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
