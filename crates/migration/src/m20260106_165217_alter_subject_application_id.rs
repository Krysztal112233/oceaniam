use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_SUBJECT_APPLICATION: &str = "fk_subjects_id_application_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Subjects::Table)
                    .add_column(uuid(Subjects::ApplicationID))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_SUBJECT_APPLICATION)
                    .from(Subjects::Table, Subjects::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_SUBJECT_APPLICATION)
                    .table(Subjects::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Subjects::Table)
                    .drop_column(Subjects::ApplicationID)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Subjects {
    Table,

    ApplicationID,
}

#[derive(DeriveIden)]
enum Applications {
    Table,

    ID,
}
