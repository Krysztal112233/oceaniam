use sea_orm_migration::prelude::*;

const IDX_USERS_APPLICATION_ID: &str = "idx_users_application_id";
const IDX_SUBJECTS_APPLICATION_ID: &str = "idx_subjects_application_id";
const IDX_KEY_BOXES_APPLICATION_ID: &str = "idx_key_boxes_application_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(IDX_USERS_APPLICATION_ID)
                    .table(Users::Table)
                    .col(Users::ApplicationID)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_SUBJECTS_APPLICATION_ID)
                    .table(Subjects::Table)
                    .col(Subjects::ApplicationID)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_KEY_BOXES_APPLICATION_ID)
                    .table(KeyBoxes::Table)
                    .col(KeyBoxes::ApplicationID)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_KEY_BOXES_APPLICATION_ID)
                    .table(KeyBoxes::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_SUBJECTS_APPLICATION_ID)
                    .table(Subjects::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_USERS_APPLICATION_ID)
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,

    ApplicationID,
}

#[derive(DeriveIden)]
enum Subjects {
    Table,

    ApplicationID,
}

#[derive(DeriveIden)]
enum KeyBoxes {
    Table,

    ApplicationID,
}
