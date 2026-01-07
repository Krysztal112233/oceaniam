use oceaniam_common::{
    config::{BackendConfig, DatabaseConfig},
    error::Error,
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

#[derive(Debug)]
pub struct PreManager {
    pub database: DatabaseConnection,
}

impl PreManager {
    pub async fn new(config: BackendConfig) -> Result<Self, Error> {
        let DatabaseConfig { dsn, .. } = config.database;

        let options = ConnectOptions::new(dsn).to_owned();

        let database = Database::connect(options).await?;

        Ok(Self { database })
    }
}
