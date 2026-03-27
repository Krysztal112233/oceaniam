use axum::http::StatusCode;
use oceaniam_common::{PageParam, PagedResponse, consts, error::Error};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder,
};
use tap::Tap;
use uuid::Uuid;

use crate::{
    helper::{PagedExecutor, PagedSelect, SafeTransactionConnectionTrait},
    model::{self, prelude::Administrators},
};

pub struct UpdateAdministratorModel {
    pub name: Option<String>,
}

#[async_trait::async_trait]
pub trait AdministratorsHelper {
    async fn get_all(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::administrators::Model>, Error> {
        use model::administrators::Column::*;

        Ok(Administrators::find()
            .order_by_asc(Name)
            .all(database)
            .await?)
    }

    async fn get_administrators(
        page: impl Into<PageParam> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::administrators::Model>, Error> {
        use model::administrators::Column::*;

        let page = page.into();

        Administrators::find()
            .order_by_asc(Name)
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    async fn get_by_id(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::administrators::Model, Error> {
        Administrators::find_by_id(id)
            .one(database)
            .await?
            .ok_or(administrator_not_found(id))
    }

    async fn get_by_name(
        name: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::administrators::Model, Error> {
        use model::administrators::Column::*;

        Ok(Administrators::find()
            .filter(Name.eq(name.into()))
            .one(database)
            .await
            .map(|it| match it {
                Some(inner) => Ok(inner),
                None => Err(Error::with_code(
                    StatusCode::NOT_FOUND,
                    consts::USER_LOGIN_FAILED_MSG,
                )),
            })??)
    }

    async fn create_administrator(
        id: Uuid,
        name: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::administrators::Model, Error> {
        Ok(model::administrators::ActiveModel {
            id: Set(id),
            name: Set(name.into()),
        }
        .insert(database)
        .await?)
    }

    async fn update_model(
        id: Uuid,

        UpdateAdministratorModel { name }: UpdateAdministratorModel,

        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::administrators::Model, Error> {
        let administrator = Self::get_by_id(id, database)
            .await?
            .into_active_model()
            .tap_mut(|it| {
                if let Some(name) = name {
                    it.name = Set(name);
                }
            });

        Ok(administrator.update(database).await?)
    }
}

impl AdministratorsHelper for Administrators {}

fn administrator_not_found(id: Uuid) -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        format!("administrator id={id} not found"),
    )
}
