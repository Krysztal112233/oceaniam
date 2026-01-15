use oceaniam_common::error::Error;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Tenants},
};

#[async_trait::async_trait]
pub trait TenantsHelper {
    async fn create(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        Ok(model::tenants::Model { id, comment: None }
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
}

impl TenantsHelper for Tenants {}
