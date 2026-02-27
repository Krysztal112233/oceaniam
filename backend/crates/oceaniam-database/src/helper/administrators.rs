use axum::http::StatusCode;
use oceaniam_common::{consts, error::Error};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Administrators},
};

#[async_trait::async_trait]
pub trait AdministratorsHelper {
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
}

impl AdministratorsHelper for Administrators {}
