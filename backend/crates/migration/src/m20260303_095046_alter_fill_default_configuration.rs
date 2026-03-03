use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"UPDATE applications
SET configuration = '{"authentication":{"issuer":"OceanIAM","audience":[]}}'::jsonb
WHERE configuration = '{}'::jsonb"#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE applications
ALTER COLUMN configuration SET DEFAULT '{"authentication":{"issuer":"OceanIAM","audience":[]}}'::jsonb"#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE applications
ALTER COLUMN configuration SET DEFAULT '{}'::jsonb"#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"UPDATE applications
SET configuration = '{}'::jsonb
WHERE configuration = '{"authentication":{"issuer":"OceanIAM","audience":[]}}'::jsonb"#,
            )
            .await?;

        Ok(())
    }
}
