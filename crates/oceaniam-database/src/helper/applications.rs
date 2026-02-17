use oceaniam_common::{PageParam, PagedResponse, error::Error};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
};
use uuid::Uuid;

use crate::{
    helper::{PagedSelect, SafeTransactionConnectionTrait},
    model::{self, prelude::Applications},
};

#[derive(Debug, Default)]
pub struct CreateApplicationOptions {
    pub comment: Option<String>,
}

#[async_trait::async_trait]
pub trait ApplicationHelper {
    async fn create(
        id: Uuid,
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        Self::create_with_opts(id, tenant_id, CreateApplicationOptions::default(), database).await
    }

    async fn create_with_opts(
        id: Uuid,
        tenant_id: Uuid,

        opts: CreateApplicationOptions,

        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        let CreateApplicationOptions { comment } = opts;

        Ok(model::applications::Model {
            id,
            tenant_id,
            comment,
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    async fn is_exist(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        Ok(Applications::find_by_id(id).one(database).await?.is_some())
    }

    async fn get_applications(
        tenant_id: Uuid,
        page: &PageParam,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::applications::Model>, Error> {
        use crate::model::applications::Column::*;

        let paginator = Applications::find()
            .filter(TenantId.eq(tenant_id))
            .paged(*page)
            .paginate(database, page.per_page);

        let items = paginator.fetch_page(0).await?;
        let total = paginator.num_items().await? as usize;
        let has_next = (page.as_offset() + items.len() as u64) < total as u64;

        Ok(PagedResponse {
            items,
            page_info: oceaniam_common::PageInfo { has_next, total },
        })
    }

    async fn delete_application(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        Applications::delete_by_id(application_id)
            .exec(database)
            .await?;

        Ok(())
    }
}

impl ApplicationHelper for Applications {}
