use axum::http::StatusCode;
use oceaniam_common::{PageParam, PagedResponse, error::Error};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    helper::{PagedSelect, SafeTransactionConnectionTrait},
    model::{self, prelude::Users},
};

#[async_trait::async_trait]
pub trait UserHelper {
    async fn create_user(
        id: impl Into<Uuid> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        todo!()
    }

    async fn get_users(
        application_id: Uuid,
        page: impl Into<PageParam> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::users::Model>, Error> {
        use crate::model::users::Column::*;

        let page = page.into();

        let paginator = Users::find()
            .filter(ApplicationId.eq(application_id))
            .paged(page)
            .paginate(database, page.per_page);

        let items = paginator.fetch_page(0).await?;
        let total = paginator.num_items().await? as usize;
        let has_next = (page.as_offset() + items.len() as u64) < total as u64;

        Ok(PagedResponse {
            items,
            page_info: oceaniam_common::PageInfo { has_next, total },
        })
    }

    async fn get_user_by_email(
        email: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        use model::users::Column::*;

        match Users::find()
            .filter(Email.eq(email.into()))
            .one(database)
            .await
        {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
            )),
            Err(e) => Err(Error::Db(e)),
        }
    }

    async fn get_user_by_phone(
        phone: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        use model::users::Column::*;

        match Users::find()
            .filter(Phone.eq(phone.into()))
            .one(database)
            .await
        {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
            )),
            Err(e) => Err(Error::Db(e)),
        }
    }
}

impl UserHelper for Users {}
