use chrono::{Duration, Utc};
use sea_orm::{FromQueryResult, Statement};
use uuid::Uuid;

use crate::error::Error;
use crate::helper::SafeTransactionConnectionTrait;

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

fn validate_granularity(g: &str) -> Result<(), Error> {
    match g {
        "day" | "week" | "month" => Ok(()),
        other => Err(Error::with_code(
            400u16,
            format!("invalid granularity: {other}, expected day/week/month"),
        )),
    }
}

pub async fn get_platform_trends(
    granularity: &str,
    range_days: u64,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<Vec<PlatformTrendRow>, Error> {
    validate_granularity(granularity)?;

    let since = Utc::now() - Duration::days(range_days as i64);

    // TODO: eliminate raw SQL by using SeaORM's query builder (Func::cust("date_trunc"))
    let sql = "SELECT date_trunc($1, bucket) AS period, \
                      entity_type, \
                      SUM(event_count)::bigint AS count \
               FROM platform_summary \
               WHERE bucket >= $2 \
               GROUP BY period, entity_type \
               ORDER BY period ASC";

    let stmt = Statement::from_sql_and_values(
        database.get_database_backend(),
        sql,
        [granularity.into(), since.into()],
    );

    let rows = database
        .query_all(stmt)
        .await?
        .into_iter()
        .map(|r| PlatformTrendRow::from_query_result(&r, ""))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

pub async fn get_application_trends(
    application_id: Uuid,
    granularity: &str,
    range_days: u64,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<Vec<ApplicationTrendRow>, Error> {
    validate_granularity(granularity)?;

    let since = Utc::now() - Duration::days(range_days as i64);

    // TODO: eliminate raw SQL by using SeaORM's query builder (Func::cust("date_trunc"))
    let sql = "SELECT date_trunc($1, bucket) AS period, \
                      SUM(event_count)::bigint AS count \
               FROM application_summary \
               WHERE application_id = $2 \
                 AND bucket >= $3 \
               GROUP BY period \
               ORDER BY period ASC";

    let stmt = Statement::from_sql_and_values(
        database.get_database_backend(),
        sql,
        [granularity.into(), application_id.into(), since.into()],
    );

    let rows = database
        .query_all(stmt)
        .await?
        .into_iter()
        .map(|r| ApplicationTrendRow::from_query_result(&r, ""))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}
