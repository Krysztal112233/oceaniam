use sea_orm_migration::prelude::*;

const PK_APPLICATION_SECRET_BINDINGS: &str = "application_secret_bindings_pkey";
const UQ_APPLICATION_SECRET_BINDINGS_SECRET_APPLICATION: &str =
    "uq_application_secret_bindings_secret_application";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .table(ApplicationSecretBindings::Table)
                    .name(UQ_APPLICATION_SECRET_BINDINGS_SECRET_APPLICATION)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"ALTER TABLE application_secret_bindings
ADD CONSTRAINT {PK_APPLICATION_SECRET_BINDINGS}
PRIMARY KEY (secret_id, application_id)"#
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"ALTER TABLE application_secret_bindings
DROP CONSTRAINT {PK_APPLICATION_SECRET_BINDINGS}"#
            ))
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

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ApplicationSecretBindings {
    Table,
    SecretId,
    ApplicationId,
}
