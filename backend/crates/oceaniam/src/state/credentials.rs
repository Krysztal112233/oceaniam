use std::{sync::Arc, time::Duration};

use axum::http::StatusCode;
use moka::future::{Cache, CacheBuilder};
use oceaniam_common::error::Error;
use oceaniam_credential::CredentialVault;
use oceaniam_database::model::{self, prelude::Credentials};
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
        self.credentials
            .try_get_with(id, async {
                Credentials::find_by_id(id)
                    .one(&self.database)
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
        Credentials::delete_by_id(id).exec(&self.database).await?;
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
    ) -> Result<model::credentials::Model, Error> {
        let vault = CredentialVault::with_password(password)?;

        let model = vault.write_to(subject_id, &self.database).await?;

        self.credentials.insert(model.id, vault).await;

        Ok(model)
    }
}
