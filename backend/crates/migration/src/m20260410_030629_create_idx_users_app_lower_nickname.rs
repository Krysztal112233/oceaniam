use sea_orm_migration::prelude::*;

const IDX_USERS_NICKNAME_TRGM: &str = "idx_users_nickname_trgm";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "CREATE INDEX IF NOT EXISTS {IDX_USERS_NICKNAME_TRGM} ON users USING gin (nickname gin_trgm_ops)"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_USERS_NICKNAME_TRGM)
                    .table(Users::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
}
