use sea_orm_migration::prelude::*;

const IDX_SUBJECTS_APPLICATION_ID: &str = "idx_subjects_application_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_SUBJECTS_APPLICATION_ID)
                    .table(Subjects::Table)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(IDX_SUBJECTS_APPLICATION_ID)
                    .table(Subjects::Table)
                    .col(Subjects::ApplicationID)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Subjects {
    Table,

    ApplicationID,
}
