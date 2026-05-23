use oceaniam_vo::pagination::{PageInfo, PageParam, PagedResponse};
use sea_orm::sea_query::extension::postgres::PgExpr;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::error::Error;
use crate::helper::SafeTransactionConnectionTrait;
use crate::model::prelude::*;
use crate::model::sea_orm_active_enums::AuditType;

/// Platform-level count statistics
pub struct PlatformCounts {
    pub total_tenants: u64,
    pub total_applications: u64,
    pub total_administrators: u64,
    pub total_application_users: u64,
    pub total_active_secrets: u64,
}

/// Application-level count statistics
pub struct ApplicationCounts {
    pub total_users: u64,
    pub total_active_keys: u64,
}

#[async_trait::async_trait]
pub trait AuditsHelper {
    async fn count_tenants(database: &impl SafeTransactionConnectionTrait) -> Result<u64, Error> {
        Ok(Tenants::find().count(database).await?)
    }

    async fn count_applications(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<u64, Error> {
        Ok(Applications::find().count(database).await?)
    }

    async fn count_administrators(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<u64, Error> {
        Ok(Administrators::find().count(database).await?)
    }

    async fn count_application_users(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<u64, Error> {
        Ok(Users::find().count(database).await?)
    }

    async fn count_active_secrets(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<u64, Error> {
        use crate::model::application_secrets::Column::*;

        Ok(ApplicationSecrets::find()
            .filter(RevokedAt.is_null())
            .count(database)
            .await?)
    }

    async fn platform_counts(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PlatformCounts, Error> {
        let (tenants, apps, admins, app_users, secrets) = futures::join!(
            Self::count_tenants(database),
            Self::count_applications(database),
            Self::count_administrators(database),
            Self::count_application_users(database),
            Self::count_active_secrets(database),
        );

        Ok(PlatformCounts {
            total_tenants: tenants?,
            total_applications: apps?,
            total_administrators: admins?,
            total_application_users: app_users?,
            total_active_secrets: secrets?,
        })
    }

    async fn application_counts(
        app_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<ApplicationCounts, Error> {
        let (users, keys) = futures::join!(
            Self::count_application_users_by_app(app_id, database),
            Self::count_active_secrets_by_app(app_id, database),
        );

        Ok(ApplicationCounts {
            total_users: users?,
            total_active_keys: keys?,
        })
    }

    /// Paginated audit log, ordered by most recent first.
    async fn get_audit_logs(
        page: PageParam,
        audit_type: Option<AuditType>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<crate::model::audits::Model>, Error> {
        use crate::model::audits::Column::*;

        let mut query = Audits::find().order_by_desc(CreatedAt);

        if let Some(atype) = audit_type {
            query = query.filter(AuditType.eq(atype));
        }

        let page = page.into_clamped();
        let paginator = query.paginate(database, page.per_page);
        let items = paginator.fetch_page(page.page.saturating_sub(1)).await?;
        let total = paginator.num_items().await? as usize;
        let has_next = (page.as_offset() + items.len() as u64) < total as u64;

        Ok(PagedResponse {
            items,
            page_info: PageInfo { has_next, total },
        })
    }

    async fn count_application_users_by_app(
        app_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<u64, Error> {
        use crate::model::users::Column::*;

        Ok(Users::find()
            .filter(ApplicationId.eq(app_id))
            .count(database)
            .await?)
    }

    async fn count_active_secrets_by_app(
        app_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<u64, Error> {
        use crate::model::application_secret_bindings::Column as BindingCol;
        use crate::model::application_secrets::Column as SecretCol;

        Ok(ApplicationSecretBindings::find()
            .filter(BindingCol::ApplicationId.eq(app_id))
            .inner_join(ApplicationSecrets)
            .filter(SecretCol::RevokedAt.is_null())
            .count(database)
            .await?)
    }

    /// Paginated audit log filtered by application, ordered by most recent first.
    async fn get_audit_logs_by_app(
        page: PageParam,
        app_id: Uuid,
        audit_type: Option<AuditType>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<crate::model::audits::Model>, Error> {
        use crate::model::audits::Column::*;
        use sea_orm::sea_query::Expr;

        let payload: serde_json::Value =
            serde_json::json!({"data": {"application_id": app_id.to_string()}});

        let mut query = Audits::find()
            .filter(Expr::col(Payload).contains(payload))
            .order_by_desc(CreatedAt);

        if let Some(atype) = audit_type {
            query = query.filter(AuditType.eq(atype));
        }

        let page = page.into_clamped();
        let paginator = query.paginate(database, page.per_page);
        let items = paginator.fetch_page(page.page.saturating_sub(1)).await?;
        let total = paginator.num_items().await? as usize;
        let has_next = (page.as_offset() + items.len() as u64) < total as u64;

        Ok(PagedResponse {
            items,
            page_info: PageInfo { has_next, total },
        })
    }
}

impl AuditsHelper for Audits {}

#[cfg(test)]
mod tests {

    use sea_orm::prelude::Expr;
    use sea_orm::{QueryTrait, Value};

    use super::*;

    /// Prefer `serde_json::Value` over `Value::Json(Some(Box::new(...)))` when passing JSON
    /// payloads to `PgExpr::contains` — the implicit conversion is equivalent.
    #[test]
    fn jsonb_contains_implicit_value_conversion_produces_same_sql() {
        use crate::model::audits::Column::*;

        let payload: serde_json::Value = serde_json::json!({"data": {"application_id": "ca189192-10a2-4af1-9f20-33ef2c4023a5".to_string()}});

        // NOTE: PREFER THIS
        let query0 = Audits::find()
            .filter(Expr::col(Payload).contains(payload.clone()))
            .order_by_desc(CreatedAt)
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();

        let query1 = Audits::find()
            .filter(Expr::col(Payload).contains(Value::Json(Some(Box::new(payload)))))
            .order_by_desc(CreatedAt)
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();

        assert_eq!(query0, query1)
    }
}
