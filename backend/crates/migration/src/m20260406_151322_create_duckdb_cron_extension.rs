use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

// NOTE: You know what, AI is actually bloody brilliant.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260406_151322_create_duckdb_cron_extension/up.sql"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260406_151322_create_duckdb_cron_extension/down.sql"
            ))
            .await?;

        Ok(())
    }
}
