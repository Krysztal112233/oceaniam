use chrono::{Duration, Utc};
use sea_orm::{
    ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Expr, Func, Order},
};
use uuid::Uuid;

use crate::error::Error;
use crate::helper::SafeTransactionConnectionTrait;
use crate::model::prelude::{ApplicationSummary, PlatformSummary};
use crate::model::{application_summary, platform_summary};

#[derive(Debug, FromQueryResult)]
pub struct PlatformTrendRow {
    pub period: chrono::DateTime<chrono::FixedOffset>,
    pub entity_type: String,
    pub count: i64,
}

#[derive(Debug, FromQueryResult)]
pub struct ApplicationTrendRow {
    pub period: chrono::DateTime<chrono::FixedOffset>,
    pub count: i64,
}

pub async fn get_platform_trends(
    granularity: &str,
    range_days: u64,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<Vec<PlatformTrendRow>, Error> {
    let since = Utc::now() - Duration::days(range_days as i64);

    let period = Expr::expr(Func::cust("date_trunc").args([
        Expr::value(granularity),
        Expr::col(platform_summary::Column::Bucket).into(),
    ]));

    let rows = PlatformSummary::find()
        .select_only()
        .column_as(period.clone(), "period")
        .column(platform_summary::Column::EntityType)
        .column_as(Expr::cust("SUM(event_count)::bigint"), "count")
        .filter(platform_summary::Column::Bucket.gte(since))
        .group_by(period.clone())
        .group_by(platform_summary::Column::EntityType)
        .order_by(period, Order::Asc)
        .into_model::<PlatformTrendRow>()
        .all(database)
        .await?;

    Ok(rows)
}

pub async fn get_application_trends(
    application_id: Uuid,
    granularity: &str,
    range_days: u64,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<Vec<ApplicationTrendRow>, Error> {
    let since = Utc::now() - Duration::days(range_days as i64);

    let period = Expr::expr(Func::cust("date_trunc").args([
        Expr::value(granularity.to_string()),
        Expr::col(application_summary::Column::Bucket).into(),
    ]));

    let rows = ApplicationSummary::find()
        .select_only()
        .column_as(period.clone(), "period")
        .column_as(Expr::cust("SUM(event_count)::bigint"), "count")
        .filter(application_summary::Column::ApplicationId.eq(application_id))
        .filter(application_summary::Column::Bucket.gte(since))
        .group_by(period.clone())
        .order_by(period, Order::Asc)
        .into_model::<ApplicationTrendRow>()
        .all(database)
        .await?;

    Ok(rows)
}
