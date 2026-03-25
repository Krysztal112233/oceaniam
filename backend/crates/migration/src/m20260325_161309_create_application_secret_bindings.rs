use sea_orm_migration::prelude::*;

const FK_APPLICATION_SECRET_BINDINGS_SECRET: &str = "fk_application_secret_bindings_secret_id";
const FK_APPLICATION_SECRET_BINDINGS_APPLICATION: &str =
    "fk_application_secret_bindings_application_id";
const UQ_APPLICATION_SECRET_BINDINGS_SECRET_APPLICATION: &str =
    "uq_application_secret_bindings_secret_application";
const IDX_APPLICATION_SECRET_BINDINGS_APPLICATION: &str =
    "idx_application_secret_bindings_application_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApplicationSecretBindings::Table)
                    .col(
                        ColumnDef::new(ApplicationSecretBindings::SecretId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ApplicationSecretBindings::ApplicationId)
                            .uuid()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_APPLICATION_SECRET_BINDINGS_SECRET)
                    .from(
                        ApplicationSecretBindings::Table,
                        ApplicationSecretBindings::SecretId,
                    )
                    .to(ApplicationSecrets::Table, ApplicationSecrets::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_APPLICATION_SECRET_BINDINGS_APPLICATION)
                    .from(
                        ApplicationSecretBindings::Table,
                        ApplicationSecretBindings::ApplicationId,
                    )
                    .to(Applications::Table, Applications::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_APPLICATION_SECRET_BINDINGS_SECRET_APPLICATION)
                    .table(ApplicationSecretBindings::Table)
                    .col(ApplicationSecretBindings::SecretId)
                    .col(ApplicationSecretBindings::ApplicationId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_APPLICATION_SECRET_BINDINGS_APPLICATION)
                    .table(ApplicationSecretBindings::Table)
                    .col(ApplicationSecretBindings::ApplicationId)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260325_161309_create_application_secret_bindings/up.sql"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ApplicationSecretBindings::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ApplicationSecretBindings {
    Table,
    SecretId,
    ApplicationId,
}

#[derive(DeriveIden)]
enum ApplicationSecrets {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    Id,
}
