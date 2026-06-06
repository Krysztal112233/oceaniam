use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Credentials},
};

#[async_trait::async_trait]
pub trait CredentialsHelper {
    async fn get_credential_by_id(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Option<model::credentials::Model>, Error> {
        Ok(Credentials::find_by_id(id).one(database).await?)
    }

    async fn delete_credential_by_id(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        Credentials::delete_by_id(id).exec(database).await?;
        Ok(())
    }

    async fn upsert_credential(
        id: Uuid,
        phc: String,
        totp: Option<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::credentials::Model, Error> {
        let active_model = model::credentials::Model { id, phc, totp }.into_active_model();

        let existing = Credentials::find_by_id(id).one(database).await?;

        Ok(match existing {
            Some(_) => Credentials::update(active_model).exec(database).await?,
            None => active_model.insert(database).await?,
        })
    }
}

impl CredentialsHelper for Credentials {}
