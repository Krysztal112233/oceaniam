use axum::extract::FromRef;
use oceaniam_common::error::Error;
use sea_orm::DatabaseConnection;

use crate::keybox::KeyBoxManager;

#[derive(Debug, Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub keybox: KeyBoxManager,
    pub _unit: (),
}

impl AppState {
    pub async fn new(database: DatabaseConnection) -> Result<Self, Error> {
        let keybox = KeyBoxManager::new(database.clone());

        Ok(Self {
            database,
            keybox,
            _unit: (),
        })
    }
}

impl FromRef<AppState> for () {
    fn from_ref(input: &AppState) -> Self {
        input._unit
    }
}
