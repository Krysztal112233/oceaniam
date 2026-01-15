use oceaniam_common::error::Error;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Tenants},
};

#[async_trait::async_trait]
pub trait TenantsHelper {
    async fn create_tenant(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::tenants::Model, Error> {
        Ok(model::tenants::Model {
            id: Uuid::now_v7(),
            comment: None,
        }
        .into_active_model()
        .insert(database)
        .await?)
    }
}

impl TenantsHelper for Tenants {}
