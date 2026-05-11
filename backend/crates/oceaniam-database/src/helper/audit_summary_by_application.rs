use chrono::{Duration, Utc};
use oceaniam_common::error::Error;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::sea_orm_active_enums::AuditType,
    view::{audit_summary_by_application, prelude::AuditSummaryByApplication},
};

#[async_trait::async_trait]
pub trait AuditSummaryByApplicationHelper {
    async fn get_last_30days_by_application(
        application_id: Uuid,
        audit_type: AuditType,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<audit_summary_by_application::Model>, Error> {
        use audit_summary_by_application::Column::*;

        let since = Utc::now().date_naive() - Duration::days(30);

        AuditSummaryByApplication::find()
            .filter(ApplicationId.eq(application_id))
            .filter(AuditType.eq(audit_type))
            .filter(Day.gte(since))
            .order_by_asc(Day)
            .all(database)
            .await
            .map_err(Into::into)
    }
}

impl AuditSummaryByApplicationHelper for AuditSummaryByApplication {}
