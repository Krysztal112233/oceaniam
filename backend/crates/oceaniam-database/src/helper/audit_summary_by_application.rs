use crate::error::Error;
use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
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

        AuditSummaryByApplication::find()
            .filter(ApplicationId.eq(application_id))
            .filter(AuditType.eq(audit_type))
            .filter(Bucket.gte(since))
            .order_by_asc(Bucket)
            .all(database)
            .await
            .map_err(Into::into)
    }
}

impl AuditSummaryByApplicationHelper for AuditSummaryByApplication {}
