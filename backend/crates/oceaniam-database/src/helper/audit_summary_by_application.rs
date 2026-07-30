use crate::error::Error;

use chrono::{Duration, Utc};
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Expr, Order},
};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{
        audit_summary_by_application, prelude::AuditSummaryByApplication,
        sea_orm_active_enums::AuditType,
    },
};

#[async_trait::async_trait]
pub trait AuditSummaryByApplicationHelper {
    /// Fetches audit event counts aggregated by day for the last 30 days.
    ///
    /// The underlying table stores data at minute-level granularity. This method
    /// re-aggregates those rows into daily buckets using `date_trunc('day', bucket)`,
    /// summing `event_count` so callers receive a daily view.
    ///
    /// # Returns
    ///
    /// A `Vec<Model>` where each element represents one day. The `bucket` field
    /// is set to `00:00:00 UTC` of that day, and `event_count` is the total
    /// number of events across all minutes within that day.
    #[tracing::instrument(
        level = "info",
        name = "db.audit_summary_by_application.get_last_30days_by_application",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_last_30days_by_application(
        application_id: Uuid,
        audit_type: AuditType,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<audit_summary_by_application::Model>, Error> {
        use audit_summary_by_application::Column::*;

        let since = (Utc::now().date_naive() - Duration::days(30))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let daily_bucket = Expr::cust("date_trunc('day', bucket)");

        AuditSummaryByApplication::find()
            .select_only()
            .column(ApplicationId)
            .column_as(daily_bucket.clone(), Bucket)
            .column(AuditType)
            .column_as(Expr::cust("SUM(event_count)::bigint"), EventCount)
            .filter(ApplicationId.eq(application_id))
            .filter(AuditType.eq(audit_type))
            .filter(Bucket.gte(since))
            .group_by(ApplicationId)
            .group_by(daily_bucket.clone())
            .group_by(AuditType)
            .order_by(daily_bucket, Order::Asc)
            .into_model::<audit_summary_by_application::Model>()
            .all(database)
            .await
            .map_err(Into::into)
    }
}

impl AuditSummaryByApplicationHelper for AuditSummaryByApplication {}
