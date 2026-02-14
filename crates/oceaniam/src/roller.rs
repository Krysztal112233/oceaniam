use oceaniam_common::{
    error::Error,
    jwks::{ManagedJwkSet, roller::ManagedJwkSetRoller},
};
use sea_orm::DatabaseConnection;

#[derive(Debug)]
pub struct BuiltinJwkSetRoller {
    database: DatabaseConnection,
}

impl BuiltinJwkSetRoller {
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }
}

#[async_trait::async_trait]
impl ManagedJwkSetRoller for BuiltinJwkSetRoller {
    async fn roll(&self) -> Result<oceaniam_common::jwks::JwkSet, Error> {
        todo!()
    }
}
