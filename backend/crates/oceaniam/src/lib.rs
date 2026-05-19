use oceaniam_common::config::DatabaseConfig;
use sea_orm::DatabaseConnection;

pub mod app;
pub mod endpoints;
pub mod error;
pub mod middlewares;
pub mod state;
pub mod util;

pub async fn setup_database(
    config: &DatabaseConfig,
) -> Result<DatabaseConnection, crate::error::Error> {
    let db = oceaniam_database::setup::connect(config).await?;
    oceaniam_database::setup::init_system(&db).await?;
    Ok(db)
}
