use oceaniam_common::error::Error;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
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
}

impl ApplicationHelper for Applications {}
