use std::time::Duration;

use moka::future::Cache;
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::applications_secrets::ApplicationSecretsHelper,
    model::{self, prelude::ApplicationSecrets},
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ManagedApplicationSecrets {
    database: DatabaseConnection,
    cache: Cache<Uuid, Vec<String>>,
}

impl ManagedApplicationSecrets {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(5))
                .build(),
        }
    }

    pub async fn get_secrets(&self, application_id: Uuid) -> Result<Vec<String>, Error> {
        Ok(self
            .cache
            .try_get_with(application_id, async {
                Ok(
                    ApplicationSecrets::get_secrets(application_id, None, &self.database)
                        .await?
                        .items
                        .into_iter()
                        .map(|it| it.secret)
                        .collect(),
                )
            })
            .await?)
    }

    pub async fn put_secret(
        &self,
        application_id: Uuid,
        secret: impl Into<String> + Send,
    ) -> Result<model::application_secrets::Model, Error> {
        let model = ApplicationSecrets::create_secret(
            application_id,
            Uuid::now_v7(),
            secret,
            &self.database,
        )
        .await?;

        self.cache.remove(&application_id).await;

        Ok(model)
    }
}
