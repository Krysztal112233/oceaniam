use axum::http::StatusCode;
use oceaniam_common::error::Error;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Users},
};

#[async_trait::async_trait]
pub trait UserHelper {
    async fn create(
        id: impl Into<Uuid> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        todo!()
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
