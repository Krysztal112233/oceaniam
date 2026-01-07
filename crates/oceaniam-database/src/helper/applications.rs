use oceaniam_common::error::Error;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self},
};

#[async_trait::async_trait]
pub trait ApplicationHelper {
    async fn create_application(
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
}
