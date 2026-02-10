use std::sync::Arc;

use axum::extract::FromRef;
use oceaniam_common::error::Error;
use sea_orm::DatabaseConnection;

use crate::keybox::KeyBoxManager;

#[derive(Debug, Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub keybox: Arc<KeyBoxManager>,
    pub _unit: (),
}

impl AppState {
    pub async fn sync_keybox(&mut self) {}

    pub async fn new(database: DatabaseConnection) -> Result<Self, Error> {
        let keybox = KeyBoxManager::new(&database).await?;

        Ok(Self {
            database,
            keybox: Arc::new(keybox),
            _unit: (),
        })
    }
}

impl FromRef<AppState> for () {
    fn from_ref(input: &AppState) -> Self {
        input._unit
    }
}
