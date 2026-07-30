use oceaniam_common::config::DatabaseConfig;
use sea_orm::DatabaseConnection;

#[tracing::instrument(
    level = "info",
    name = "db.setup",
    skip_all,
    fields(otel.kind = "internal")
)]
pub async fn setup_database(
    config: &DatabaseConfig,
) -> Result<DatabaseConnection, crate::error::Error> {
    let db = oceaniam_database::setup::connect(config).await?;
    oceaniam_database::setup::init_system(&db).await?;
    Ok(db)
}
