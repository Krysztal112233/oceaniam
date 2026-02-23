use std::time::Duration;

use axum::http::StatusCode;
use moka::future::Cache;
use oceaniam_common::{error::Error, helpers::gen_random_with_charset};
use oceaniam_database::{
    helper::applications_secrets::ApplicationSecretsHelper,
    model::{self, prelude::ApplicationSecrets},
};
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

type SecretModel = model::application_secrets::Model;

#[derive(Debug, Clone)]
pub struct ManagedApplicationSecrets {
    database: DatabaseConnection,
    application_secrets: Cache<Uuid, Vec<SecretModel>>,
    secret_of_application: Cache<String, Uuid>,
    secret_id_of_application: Cache<Uuid, Uuid>,
}

impl ManagedApplicationSecrets {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            application_secrets: Cache::builder()
                .time_to_live(Duration::from_secs(5))
                .build(),
            secret_of_application: Cache::builder()
                .time_to_live(Duration::from_secs(2))
                .max_capacity(1024)
                .build(),
            secret_id_of_application: Cache::builder()
                .time_to_live(Duration::from_secs(2))
                .max_capacity(1024)
                .build(),
        }
    }

    pub async fn get_secrets(&self, application_id: Uuid) -> Result<Vec<SecretModel>, Error> {
        Ok(self
            .application_secrets
            .try_get_with(application_id, async {
                Ok(
                    ApplicationSecrets::get_secrets(application_id, None, &self.database)
                        .await?
                        .items,
                )
            })
            .await?)
    }

    pub async fn put_secret(&self, application_id: Uuid) -> Result<SecretModel, Error> {
        self.put_secret_with_generated(application_id, gen_secret().await)
            .await
    }

    pub async fn put_secret_with_generated(
        &self,
        application_id: Uuid,
        secret: impl Into<String> + Send,
    ) -> Result<SecretModel, Error> {
        let secret: String = secret.into();

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

    pub async fn find_secret_belong_by_id(&self, secret_id: Uuid) -> Result<Uuid, Error> {
        Ok(self
            .secret_id_of_application
            .try_get_with(secret_id, async {
                ApplicationSecrets::find_by_id(secret_id)
                    .one(&self.database)
                    .await?
                    .map(|it| it.application_id)
                    .ok_or(Error::with_code(
                        StatusCode::NOT_FOUND,
                        format!("cannot found secret_id={secret_id}"),
                    ))
            })
            .await?)
    }

    async fn invalidate_secret(&self, secret_id: Uuid) -> Result<(), Error> {
        let application_id = self.find_secret_belong_by_id(secret_id).await?;

        self.application_secrets.remove(&application_id).await;
        self.secret_id_of_application.remove(&secret_id).await;

        Ok(())
    }

    pub async fn drop_secret(&self, secret_id: Uuid) -> Result<(), Error> {
        self.invalidate_secret(secret_id).await?;
        ApplicationSecrets::delete_by_id(secret_id)
            .exec(&self.database)
            .await?;

        Ok(())
    }
}

async fn gen_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

    let random = gen_random_with_charset(32, CHARSET);

    format!("app_{random}")
}
