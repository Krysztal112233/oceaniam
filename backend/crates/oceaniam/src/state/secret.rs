use std::{collections::HashMap, time::Duration};

use crate::error::Error;
use axum::http::StatusCode;
use moka::future::Cache;
use oceaniam_api::{PageParam, PagedResponse};
use oceaniam_common::helpers::gen_random_with_charset;
use oceaniam_database::{
    helper::{applications::ApplicationHelper, applications_secrets::ApplicationSecretsHelper},
    model::{
        application_secrets::Model as SecretModel,
        prelude::{ApplicationSecrets, Applications},
    },
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

/// If [crate::state::applications::ApplicationUsers::application_id] doesn't existed in database, the entire functions of this
/// structure will be noop
///
/// TODO: In future, using [xorf] to detect does secret existed in database for higher performance.
#[derive(Debug, Clone)]
struct SecretCaches {
    by_application: Cache<Uuid, Vec<SecretModel>>,
    by_id: Cache<Uuid, SecretModel>,
    application_ids_by_secret_id: Cache<Uuid, Vec<Uuid>>,
    application_ids_by_secret: Cache<String, Vec<Uuid>>,
}

impl SecretCaches {
    fn new() -> Self {
        Self {
            by_application: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
            by_id: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
            application_ids_by_secret_id: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
            application_ids_by_secret: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
        }
    }
}

/// Manages application `app_xxx` secrets used by external applications to authenticate against the
/// OceanIAM API.
///
/// Provides caching via [`SecretCaches`] for secrets.
#[derive(Debug, Clone)]
pub struct Secrets {
    database: DatabaseConnection,
    caches: SecretCaches,
}

impl Secrets {
    /// Creates a new [`Secrets`] manager with the given database connection.
    pub fn new(database: DatabaseConnection) -> Secrets {
        Secrets {
            database,
            caches: SecretCaches::new(),
        }
    }

    /// Generates a new `app_xxx` secret and persists it as an unbound secret (not yet associated
    /// with any application).
    pub async fn create_secret(&self) -> Result<SecretModel, Error> {
        let model =
            ApplicationSecrets::create_secret_unbound(Uuid::now_v7(), gen_secret(), &self.database)
                .await?;

        self.cache_secret_model(&model).await;
        self.cache_secret_application_ids(model.id, Vec::new())
            .await;

        Ok(model)
    }

    /// Loads every known [`SecretModel`] from the database and populates the `by_id` cache.
    ///
    /// Prefer [`get_secret_models`](Self::get_secret_models) for paginated access.
    pub async fn get_all_secrets(&self) -> Result<Vec<SecretModel>, Error> {
        let models = ApplicationSecrets::get_all_secret_models(&self.database).await?;

        for model in &models {
            self.cache_secret_model(model).await;
        }

        Ok(models)
    }

    /// Returns a paginated view of all secrets, warming the `by_id` cache along the way.
    pub async fn get_secret_models(
        &self,
        page: PageParam,
    ) -> Result<PagedResponse<SecretModel>, Error> {
        let paged = ApplicationSecrets::get_secret_models(page, &self.database).await?;

        for model in &paged.items {
            self.cache_secret_model(model).await;
        }

        Ok(paged)
    }

    /// Returns a single [`SecretModel`] by its ID, backed by the `by_id` cache.
    pub async fn get_secret(&self, secret_id: Uuid) -> Result<SecretModel, Error> {
        Ok(self
            .caches
            .by_id
            .try_get_with(secret_id, async {
                ApplicationSecrets::get_secret(secret_id, &self.database)
                    .await
                    .map_err(Into::into)
            })
            .await?)
    }

    /// Returns the list of application IDs that the given secret is bound to, backed by the
    /// `application_ids_by_secret_id` cache.
    pub async fn get_secret_application_ids(&self, secret_id: Uuid) -> Result<Vec<Uuid>, Error> {
        Ok(self
            .caches
            .application_ids_by_secret_id
            .try_get_with(secret_id, async {
                ApplicationSecrets::get_application_ids_of_secret(secret_id, &self.database)
                    .await
                    .map_err(Into::into)
            })
            .await?)
    }

    /// Returns the full application-ID-to-secret-IDs map for every secret in the database.
    ///
    /// Skips caching because the result is already a complete snapshot.
    pub async fn get_secret_application_ids_batch(
        &self,
    ) -> Result<HashMap<Uuid, Vec<Uuid>>, Error> {
        ApplicationSecrets::get_application_ids_grouped_by_secret_ids(None, &self.database)
            .await
            .map_err(Into::into)
    }

    /// Returns the application-ID-to-secret-IDs map for the specified secrets, and populates the
    /// `application_ids_by_secret_id` cache for each one.
    pub async fn get_secret_application_ids_batch_by_ids(
        &self,
        secret_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, Vec<Uuid>>, Error> {
        let application_ids_by_secret =
            ApplicationSecrets::get_application_ids_grouped_by_secret_ids(
                Some(&secret_ids),
                &self.database,
            )
            .await?;

        for (secret_id, application_ids) in &application_ids_by_secret {
            self.cache_secret_application_ids(*secret_id, application_ids.clone())
                .await;
        }

        Ok(application_ids_by_secret)
    }

    /// Reloads the `by_application` cache entry for the given application from the database.
    async fn refresh(&self, application_id: Uuid) -> Result<(), Error> {
        self.is_application_exist(application_id).await?;

        self.caches
            .by_application
            .insert(
                application_id,
                ApplicationSecrets::get_all_secrets_of(application_id, &self.database).await?,
            )
            .await;

        Ok(())
    }

    /// Resolves the application IDs that a given secret string (e.g. `app_xxx`) is valid for,
    /// backed by the `application_ids_by_secret` cache.
    ///
    /// Returns a [`NOT_FOUND`](StatusCode::NOT_FOUND) error when the secret value does not exist.
    pub async fn find_secret_belong_to(
        &self,
        secret: impl Into<String>,
    ) -> Result<Vec<Uuid>, Error> {
        let secret = secret.into();

        Ok(self
            .caches
            .application_ids_by_secret
            .try_get_with(secret.clone(), async {
                ApplicationSecrets::find_secret_can_be_used_for(secret, &self.database)
                    .await
                    .map_err(Into::into)
            })
            .await?)
    }

    /// Returns all secrets bound to the given application, backed by the `by_application` cache.
    pub async fn get_all_secrets_of(
        &self,
        application_id: Uuid,
    ) -> Result<Vec<SecretModel>, Error> {
        self.is_application_exist(application_id).await?;

        Ok(self
            .caches
            .by_application
            .try_get_with(application_id, async {
                ApplicationSecrets::get_all_secrets_of(application_id, &self.database)
                    .await
                    .map_err(Into::into)
            })
            .await?)
    }

    /// Unbinds a secret from an application and refreshes the application's secret cache.
    pub async fn delete_secret(&self, application_id: Uuid, secret_id: Uuid) -> Result<(), Error> {
        self.is_application_exist(application_id).await?;
        self.is_secret_id_exist(secret_id).await?;

        ApplicationSecrets::delete_secret(application_id, secret_id, &self.database).await?;

        self.refresh(application_id).await?;
        self.invalidate_secret_bindings(secret_id).await;

        Ok(())
    }

    /// Globally deletes a secret by its ID, removing it from every application it was bound to,
    /// and invalidates all related caches.
    pub async fn delete_secret_by_id(&self, secret_id: Uuid) -> Result<(), Error> {
        self.is_secret_id_exist(secret_id).await?;

        let application_ids =
            ApplicationSecrets::get_application_ids_of_secret(secret_id, &self.database).await?;

        ApplicationSecrets::delete_secret_by_id(secret_id, &self.database).await?;

        for application_id in application_ids {
            self.invalidate_application_secrets(application_id).await;
        }

        self.invalidate_secret(secret_id).await;

        Ok(())
    }

    /// Fast existence check for a secret ID via database query.
    async fn is_secret_id_exist(&self, secret_id: Uuid) -> Result<(), Error> {
        ApplicationSecrets::get_secret(secret_id, &self.database).await?;
        Ok(())
    }

    /// Fast existence check for an application ID via database query.
    async fn is_application_exist(&self, application_id: Uuid) -> Result<(), Error> {
        if Applications::is_exist(application_id, &self.database).await? {
            Ok(())
        } else {
            Err(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("application_id={application_id} doesn't exist"),
            ))
        }
    }

    /// Writes a [`SecretModel`] into the `by_id` cache.
    async fn cache_secret_model(&self, model: &SecretModel) {
        self.caches.by_id.insert(model.id, model.clone()).await;
    }

    /// Writes the application-ID list for a secret into the `application_ids_by_secret_id` cache.
    async fn cache_secret_application_ids(&self, secret_id: Uuid, application_ids: Vec<Uuid>) {
        self.caches
            .application_ids_by_secret_id
            .insert(secret_id, application_ids)
            .await;
    }

    /// Evicts the `application_ids_by_secret_id` entry and the entire `application_ids_by_secret`
    /// cache (cheaper than selective invalidation).
    async fn invalidate_secret_bindings(&self, secret_id: Uuid) {
        self.caches
            .application_ids_by_secret_id
            .invalidate(&secret_id)
            .await;

        self.caches.application_ids_by_secret.invalidate_all();
    }

    /// Evicts a secret from `by_id` and all of its binding caches.
    async fn invalidate_secret(&self, secret_id: Uuid) {
        self.caches.by_id.invalidate(&secret_id).await;

        self.invalidate_secret_bindings(secret_id).await;
    }

    /// Evicts the `by_application` cache entry for the given application.
    async fn invalidate_application_secrets(&self, application_id: Uuid) {
        self.caches.by_application.invalidate(&application_id).await;
    }
}

/// Generates a random `app_xxx`-prefixed secret string (32 alphanumeric chars after the prefix).
fn gen_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

    let random = gen_random_with_charset(32, CHARSET);

    format!("app_{random}")
}
