use std::{sync::Arc, time::Duration};

use argon2::Argon2;
use axum::http::StatusCode;
use moka::future::{Cache, CacheBuilder};
use oceaniam_common::error::Error;
use oceaniam_credential::CredentialVault;
use oceaniam_database::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Credentials},
};
use sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ManagedCredentialVaults {
    database: DatabaseConnection,

    credentials: Cache<Uuid, CredentialVault>,
}

impl ManagedCredentialVaults {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            credentials: CacheBuilder::default()
                .max_capacity(102400)
                .time_to_live(Duration::from_mins(30))
                .build(),
        }
    }

    pub async fn get_credential(&self, id: Uuid) -> Result<CredentialVault, Arc<Error>> {
        self.get_credential_in_tx(id, &self.database).await
    }

    pub async fn get_credential_in_tx(
        &self,
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<CredentialVault, Arc<Error>> {
        self.credentials
            .try_get_with(id, async {
                Credentials::find_by_id(id)
                    .one(database)
                    .await
                    .inspect_err(|e| error!("{e}"))?
                    .ok_or(Error::with_code(
                        StatusCode::NOT_FOUND,
                        format!("cannot find {} in database", id),
                    ))
                    .map(CredentialVault::from)
            })
            .await
    }

    /// Permanently deletes a credential by its ID.
    ///
    /// This method removes the credential from both the database and the in-memory cache.
    /// Use with caution as this operation cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the credential to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the database delete operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the database delete operation fails (e.g., credential not found).
    pub async fn drop_credential(&self, id: Uuid) -> Result<(), Error> {
        self.drop_credential_in_tx(id, &self.database).await
    }

    pub async fn drop_credential_in_tx(
        &self,
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        Credentials::delete_by_id(id).exec(database).await?;
        self.credentials.remove(&id).await;

        Ok(())
    }

    /// Creates a new credential with the given password.
    ///
    /// This method initializes a [CredentialVault] using the provided password,
    /// persists it to the database with the specified subject ID, and caches
    /// the vault in memory for subsequent operations.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier (subject ID) to associate with this credential
    /// * `password` - The password to hash and store securely
    ///
    /// # Returns
    ///
    /// Returns the created credential model on success, or an error if the
    /// password hashing or database write fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The password cannot be hashed (e.g., invalid format)
    /// - The database write operation fails
    pub async fn create_with_password(
        &self,
        subject_id: Uuid,
        password: impl AsRef<str> + Send,
        argon2: &Argon2<'_>,
    ) -> Result<model::credentials::Model, Error> {
        self.create_with_password_in_tx(subject_id, password, argon2, &self.database)
            .await
    }

    pub async fn create_with_password_in_tx(
        &self,
        subject_id: Uuid,
        password: impl AsRef<str> + Send,
        argon2: &Argon2<'_>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::credentials::Model, Error> {
        let vault = CredentialVault::with_password(password, argon2)?;

        let model = vault.write_to(subject_id, database).await?;

        self.credentials.insert(model.id, vault).await;

        Ok(model)
    }

    pub async fn update_password(
        &self,
        subject_id: Uuid,
        password: impl AsRef<str> + Send,
        argon2: &Argon2<'_>,
    ) -> Result<model::credentials::Model, Error> {
        self.update_password_in_tx(subject_id, password, argon2, &self.database)
            .await
    }

    pub async fn update_password_in_tx(
        &self,
        subject_id: Uuid,
        password: impl AsRef<str> + Send,
        argon2: &Argon2<'_>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::credentials::Model, Error> {
        Ok(self
            .get_credential_in_tx(subject_id, database)
            .await?
            .update_password(password, argon2)?
            .write_to(subject_id, database)
            .await?)
    }
}
