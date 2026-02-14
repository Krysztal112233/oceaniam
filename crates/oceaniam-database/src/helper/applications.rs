use oceaniam_common::{PageParam, PagedResponse, error::Error};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
};
use uuid::Uuid;

use crate::{
    helper::{PagedSelect, SafeTransactionConnectionTrait},
    model::{self, prelude::Applications},
};

#[async_trait::async_trait]
pub trait ApplicationHelper {
    async fn create(
        id: Uuid,
        tenants_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        Ok(model::applications::Model {
            id,
            tenants_id,
            comment: None,
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
        database: &impl SafeTransactionConnectionTrait,
        tenants_id: Uuid,
        page: &PageParam,
    ) -> Result<PagedResponse<model::applications::Model>, Error> {
        use crate::model::applications::Column::*;

        let paginator = Applications::find()
            .filter(TenantsId.eq(tenants_id))
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
}

impl ApplicationHelper for Applications {}
