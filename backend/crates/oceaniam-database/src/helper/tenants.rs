use crate::error::Error;
use oceaniam_common::consts;
use oceaniam_vo::pagination::{PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    sea_query::Expr,
};
use uuid::Uuid;

use crate::{
    helper::{PagedExecutor, PagedSelect, SafeTransactionConnectionTrait, exactly_one_returned},
    model::{self, prelude::Tenants},
};

#[async_trait::async_trait]
pub trait TenantsHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.tenants.get_tenant",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_tenant(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        crate::system_protected_get!(
            Tenants,
            consts::SYSTEM_TENANT_UUID,
            id,
            database,
            tenant_not_found
        )
    }

    #[tracing::instrument(
        level = "info",
        name = "db.tenants.create_tenant",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_tenant(
        id: Uuid,
        comment: Option<impl Into<String> + Send>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        Ok(model::tenants::Model {
            id,
            comment: comment.map(Into::into),
            created_at: crate::helper::db_clock_now(database).await?,
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.tenants.is_exist",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn is_exist(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        crate::system_protected_is_exist!(Tenants, consts::SYSTEM_TENANT_UUID, id, database)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.tenants.is_system_tenant_exist",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn is_system_tenant_exist(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        crate::system_protected_is_system_exist!(Tenants, consts::SYSTEM_TENANT_UUID, database)
    }

    /// Probes whether a tenant row exists, including the reserved system tenant.
    ///
    /// Unlike [`Self::is_exist`] this deliberately does not exclude the system tenant; it
    /// exists for internal callers (e.g. keybox auto-bootstrap) that must serve the legacy
    /// system tenant too.
    #[tracing::instrument(
        level = "info",
        name = "db.tenants.tenant_exists",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn tenant_exists(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        Ok(Tenants::find_by_id(id).one(database).await?.is_some())
    }

    #[tracing::instrument(
        level = "info",
        name = "db.tenants.delete_tenant",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn delete_tenant(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        crate::system_protected_delete!(
            Tenants,
            consts::SYSTEM_TENANT_UUID,
            id,
            database,
            tenant_not_found
        )
    }

    /// Updates a tenant's comment.
    ///
    /// The write is a primary-key-filtered conditional update that keeps the system tenant
    /// excluded and reports [`tenant_not_found`] when the row is gone instead of silently
    /// succeeding.
    #[tracing::instrument(
        level = "info",
        name = "db.tenants.update_comment",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn update_comment(
        id: Uuid,
        comment: Option<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        use model::tenants::Column::*;

        let updated = Tenants::update_many()
            .filter(Id.eq(id))
            .filter(Id.ne(consts::SYSTEM_TENANT_UUID))
            .col_expr(Comment, Expr::value(comment))
            .exec_with_returning(database)
            .await?;

        exactly_one_returned(updated, || tenant_not_found(id))
    }

    /// NOTE: This helper intentionally excludes the system tenant from regular
    /// tenant listing.
    #[tracing::instrument(
        level = "info",
        name = "db.tenants.get_tenants",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_tenants(
        page: Option<PageParam>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::tenants::Model>, Error> {
        use crate::model::tenants::Column::*;

        let query = Tenants::find().filter(Id.ne(consts::SYSTEM_TENANT_UUID));

        let Some(page) = page else {
            return query
                .all(database)
                .await
                .map(PagedResponse::with_entire)
                .map_err(Into::into);
        };

        query
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    #[tracing::instrument(
        level = "info",
        name = "db.tenants.list_all_tenants",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn list_all_tenants(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::tenants::Model>, Error> {
        use crate::model::tenants::Column::*;

        Ok(Tenants::find()
            .filter(Id.ne(consts::SYSTEM_TENANT_UUID))
            .all(database)
            .await?)
    }

    /// NOTE: This is the explicit escape hatch for internal code that still
    /// needs to access the reserved system tenant record.
    #[tracing::instrument(
        level = "info",
        name = "db.tenants.get_system_tenant",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_system_tenant(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        Tenants::find_by_id(consts::SYSTEM_TENANT_UUID)
            .one(database)
            .await?
            .ok_or(tenant_not_found(consts::SYSTEM_TENANT_UUID))
    }
}

impl TenantsHelper for Tenants {}

fn tenant_not_found(tenant_id: Uuid) -> Error {
    Error::not_found(format!("tenants id={tenant_id} not found"))
}
