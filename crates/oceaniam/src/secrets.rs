use std::time::Duration;

use moka::future::Cache;
use oceaniam_common::{error::Error, helpers::gen_random_with_charset};
use oceaniam_database::{
    helper::applications_secrets::ApplicationSecretsHelper,
    model::{self, prelude::ApplicationSecrets},
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ManagedApplicationSecrets {
    database: DatabaseConnection,
    application_secrets: Cache<Uuid, Vec<String>>,
    secret_of_application: Cache<String, Uuid>,
}

impl ManagedApplicationSecrets {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            application_secrets: Cache::builder()
                .time_to_live(Duration::from_secs(5))
                .build(),
            secret_of_application: Cache::builder().max_capacity(102400).build(),
        }
    }

    pub async fn get_secrets(&self, application_id: Uuid) -> Result<Vec<String>, Error> {
        Ok(self
            .application_secrets
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
    ) -> Result<model::application_secrets::Model, Error> {
        let secret = gen_secret().await;

        let model = ApplicationSecrets::create_secret(
            application_id,
            Uuid::now_v7(),
            secret.clone(),
            &self.database,
        )
        .await?;

        self.application_secrets.remove(&application_id).await;
        self.secret_of_application
            .insert(secret, application_id)
            .await;

        Ok(model)
    }

    pub async fn find_secret_belong(
        &self,
        secret: impl Into<String> + Send + Sync,
    ) -> Result<Uuid, Error> {
        let secret = secret.into();
        Ok(self
            .secret_of_application
            .try_get_with(secret.clone(), async {
                ApplicationSecrets::find_secret_belong(secret, &self.database)
                    .await
                    .map(|it| it.application_id)
            })
            .await?)
    }
}

async fn gen_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

    let random = gen_random_with_charset(32, CHARSET);

    format!("app_{random}")
}
