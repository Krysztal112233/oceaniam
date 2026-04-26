use sea_orm_migration::prelude::*;

const IDX_CHALLENGES_APPLICATION_ID_ID_SUBJECT_ID: &str =
    "idx_challenges_application_id_id_subject_id";
const IDX_CHALLENGES_APPLICATION_ID_ID: &str = "idx_challenges_application_id_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_CHALLENGES_APPLICATION_ID_ID_SUBJECT_ID)
                    .table(Challenges::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_CHALLENGES_APPLICATION_ID_ID)
                    .table(Challenges::Table)
                    .if_not_exists()
                    .col(Challenges::ApplicationId)
                    .col(Challenges::Id)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_CHALLENGES_APPLICATION_ID_ID)
                    .table(Challenges::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_CHALLENGES_APPLICATION_ID_ID_SUBJECT_ID)
                    .table(Challenges::Table)
                    .if_not_exists()
                    .col(Challenges::ApplicationId)
                    .col(Challenges::Id)
                    .col(Challenges::SubjectId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Challenges {
    Table,
    Id,
    ApplicationId,
    SubjectId,
}
