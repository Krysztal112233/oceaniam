use crate::error::Error;
use axum::http::StatusCode;
use oceaniam_common::consts;
use oceaniam_vo::pagination::{PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter,
};
use uuid::Uuid;

use crate::{
    helper::{PagedExecutor, PagedSelect, SafeTransactionConnectionTrait},
    model::{self, prelude::Tenants},
};

#[async_trait::async_trait]
pub trait TenantsHelper {
    /// NOTE: This helper intentionally blocks direct reads of the system tenant
    /// from the regular tenant access path.
    async fn get_tenant(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        if is_system_tenant(id) {
            return Err(tenant_not_found(id));
        }

        Tenants::find_by_id(id)
            .one(database)
            .await?
            .ok_or(tenant_not_found(id))
    }

    async fn create_tenant(
        id: Uuid,
        comment: Option<impl Into<String> + Send>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        Ok(model::tenants::Model {
            id,
            comment: comment.map(Into::into),
            created_at: chrono::Utc::now().into(),
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    async fn is_exist(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        if is_system_tenant(id) {
            return Ok(false);
        }

        Ok(Tenants::find_by_id(id).one(database).await?.is_some())
    }

    /// NOTE: This helper is reserved for internal bootstrap and system-only
    /// code paths that need the real existence of the reserved system tenant.
    async fn is_system_tenant_exist(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        Ok(Tenants::find_by_id(consts::SYSTEM_TENANT_UUID)
            .one(database)
            .await?
            .is_some())
    }

    /// NOTE: This helper intentionally blocks deletion of the reserved system
    /// tenant through the regular tenant lifecycle path.
    async fn delete_tenant(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        if is_system_tenant(id) {
            return Err(tenant_not_found(id));
        }

        Tenants::delete_by_id(id).exec(database).await?;
        Ok(())
    }

    async fn update_comment(
        id: Uuid,
        comment: Option<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        let mut tenant = Self::get_tenant(id, database).await?.into_active_model();
        tenant.comment = Set(comment);

        Ok(tenant.update(database).await?)
    }

    /// NOTE: This helper intentionally excludes the system tenant from regular
    /// tenant listing.
    async fn get_tenants(
        page: impl Into<PageParam> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::tenants::Model>, Error> {
        use crate::model::tenants::Column::*;

        let page = page.into();

        Tenants::find()
            .filter(Id.ne(consts::SYSTEM_TENANT_UUID))
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    /// NOTE: This helper intentionally excludes the system tenant from regular
    /// tenant listing.
    async fn get_all_tenants(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::tenants::Model>, Error> {
        use crate::model::tenants::Column::*;

        Ok(Tenants::find()
            .filter(Id.ne(consts::SYSTEM_TENANT_UUID))
            .all(database)
            .await
            .map(PagedResponse::with_entire)?)
    }

    /// NOTE: This is the explicit escape hatch for internal code that still
    /// needs to access the reserved system tenant record.
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

fn is_system_tenant(tenant_id: Uuid) -> bool {
    tenant_id == consts::SYSTEM_TENANT_UUID
}

fn tenant_not_found(tenant_id: Uuid) -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        format!("tenants id={tenant_id} not found"),
    )
}
