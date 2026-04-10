use sea_orm_migration::prelude::*;

const IDX_SUBJECTS_APPLICATION_ID_CREATED_AT_ID_DESC: &str =
    "idx_subjects_application_id_created_at_id_desc";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(IDX_SUBJECTS_APPLICATION_ID_CREATED_AT_ID_DESC)
                    .table(Subjects::Table)
                    .if_not_exists()
                    .col(Subjects::ApplicationID)
                    .col((Subjects::CreatedAt, IndexOrder::Desc))
                    .col((Subjects::Id, IndexOrder::Desc))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_SUBJECTS_APPLICATION_ID_CREATED_AT_ID_DESC)
                    .table(Subjects::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Subjects {
    Table,
    Id,
    ApplicationID,
    CreatedAt,
}
