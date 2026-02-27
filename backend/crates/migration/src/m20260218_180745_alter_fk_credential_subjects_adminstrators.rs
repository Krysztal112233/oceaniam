use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_SUBJECTS_CREDENTIALS: &str = "fk_subjects_credentials_id";
const FK_ADMINISTRATORS_CREDENTIALS: &str = "fk_administrators_credentials_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add foreign key: subjects.id -> credentials.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Subjects::Table, Subjects::Id)
                    .to(Credentials::Table, Credentials::Id)
                    .name(FK_SUBJECTS_CREDENTIALS)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add foreign key: administrators.id -> credentials.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Administrators::Table, Administrators::Id)
                    .to(Credentials::Table, Credentials::Id)
                    .name(FK_ADMINISTRATORS_CREDENTIALS)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign keys in reverse order of creation
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Administrators::Table)
                    .name(FK_ADMINISTRATORS_CREDENTIALS)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Subjects::Table)
                    .name(FK_SUBJECTS_CREDENTIALS)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Subjects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Administrators {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Credentials {
    Table,
    Id,
}
