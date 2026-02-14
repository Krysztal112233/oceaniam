use axum::extract::FromRef;
use oceaniam_common::{error::Error, jwks::ManagedJwkSet};
use sea_orm::DatabaseConnection;

use crate::{keybox::ApplicationKeyBoxManager, roller::BuiltinJwkSetRoller};

#[derive(Debug, Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub keybox: ApplicationKeyBoxManager,
    pub system_jwks: ManagedJwkSet,
    pub _unit: (),
}

impl AppState {
    pub async fn new(database: DatabaseConnection) -> Result<Self, Error> {
        let keybox = ApplicationKeyBoxManager::new(database.clone());

        Ok(Self {
            database: database.clone(),
            keybox,
            system_jwks: ManagedJwkSet::new(BuiltinJwkSetRoller::new(database)).await?,
            _unit: (),
        })
    }
}

impl FromRef<AppState> for () {
    fn from_ref(input: &AppState) -> Self {
        input._unit
    }
}
