use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20261104_111941_extract_key_box_kek_version/up.sql"
            ))
            .await?;

        Ok(())
    }

    /// Backup-only by deployment policy: new rows store the KEK version only in
    /// `key_boxes.kek_version`, and supported rollback means restoring a backup.
    ///
    /// Down-side policy exception approved on 2026-11-06. SQL could reconstruct the
    /// legacy JSON field, but the owner chooses not to maintain that rollback format.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "irreversible: m20261104_111941_extract_key_box_kek_version is backup-only by \
             deployment policy; restore from a database backup"
                .to_owned(),
        ))
    }
}
