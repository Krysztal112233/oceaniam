use axum::http::StatusCode;
use oceaniam_common::{consts, error::Error};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Administrators},
};

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

    async fn create(
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
}

impl AdministratorsHelper for Administrators {}

fn administrator_not_found(id: Uuid) -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        format!("administrator id={id} not found"),
    )
}
