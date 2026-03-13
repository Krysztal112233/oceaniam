use axum::http::StatusCode;
use oceaniam_common::{PageParam, PagedResponse, error::Error};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, PaginatorTrait};
use uuid::Uuid;

use crate::{
    helper::{PagedExecutor, PagedSelect, SafeTransactionConnectionTrait},
    model::{self, prelude::Tenants},
};

#[async_trait::async_trait]
pub trait TenantsHelper {
    async fn get_tenant(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        Tenants::find_by_id(id)
            .one(database)
            .await?
            .ok_or(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("tenants id={id} not found"),
            ))
    }

    async fn create_tenant(
        id: Uuid,
        comment: Option<impl Into<String> + Send>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        Ok(model::tenants::Model {
            id,
            comment: comment.map(Into::into),
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    async fn is_exist(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        Ok(Tenants::find_by_id(id).one(database).await?.is_some())
    }

    async fn delete_tenant(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        Tenants::delete_by_id(id).exec(database).await?;
        Ok(())
    }

    async fn get_tenants(
        page: impl Into<PageParam> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::tenants::Model>, Error> {
        let page = page.into();

        Tenants::find()
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    async fn get_all_tenants(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::tenants::Model>, Error> {
        Ok(Tenants::find()
            .all(database)
            .await
            .map(PagedResponse::with_entire)?)
    }
}

impl TenantsHelper for Tenants {}
