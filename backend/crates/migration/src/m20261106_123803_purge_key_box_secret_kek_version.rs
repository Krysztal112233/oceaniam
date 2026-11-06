use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20261106_123803_purge_key_box_secret_kek_version/up.sql"
            ))
            .await?;

        Ok(())
    }

    /// Backup-only by deployment policy. SQL could restore the JSON field from
    /// `key_boxes.kek_version`, but the owner chooses not to maintain that rollback format.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "irreversible: m20261106_123803_purge_key_box_secret_kek_version is backup-only by \
             deployment policy; restore from a database backup"
                .to_owned(),
        ))
    }
}
