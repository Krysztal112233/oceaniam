use std::{sync::Arc, time::Duration};

use axum::http::StatusCode;
use log::error;
use moka::future::{Cache, CacheBuilder};
use oceaniam_common::error::Error;
use oceaniam_credential::CredentialVault;
use oceaniam_database::model::prelude::Credentials;
use sea_orm::{DatabaseConnection, EntityTrait};
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

    pub async fn get_credential(
        &self,
        id: impl Into<Uuid> + Send + Clone,
    ) -> Result<CredentialVault, Arc<Error>> {
        let id = id.into();

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

    pub async fn remove_credential(&self, id: impl Into<Uuid> + Send) {
        self.credentials.remove(&id.into()).await;
    }
}
