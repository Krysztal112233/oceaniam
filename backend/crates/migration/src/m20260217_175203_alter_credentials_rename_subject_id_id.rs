use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // rename subject_id to id
        manager
            .alter_table(
                Table::alter()
                    .table(Credentials::Table)
                    .rename_column(Credentials::SubjectId, Credentials::Id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // rename id to subject_id
        manager
            .alter_table(
                Table::alter()
                    .table(Credentials::Table)
                    .rename_column(Credentials::Id, Credentials::SubjectId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Credentials {
    Table,

    Id,
    SubjectId,
}
