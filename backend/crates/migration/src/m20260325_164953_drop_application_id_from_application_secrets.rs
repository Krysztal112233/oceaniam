use sea_orm_migration::prelude::*;

const FK_APPLICATION_SECRET_APPLICATION: &str = "fk_application_secret_application_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_APPLICATION_SECRET_APPLICATION)
                    .table(ApplicationSecrets::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ApplicationSecrets::Table)
                    .drop_column(ApplicationSecrets::ApplicationId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ApplicationSecrets::Table)
                    .add_column(
                        ColumnDef::new(ApplicationSecrets::ApplicationId)
                            .uuid()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260325_164953_drop_application_id_from_application_secrets/down.sql"
            ))
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ApplicationSecrets::Table)
                    .modify_column(
                        ColumnDef::new(ApplicationSecrets::ApplicationId)
                            .uuid()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_APPLICATION_SECRET_APPLICATION)
                    .from(ApplicationSecrets::Table, ApplicationSecrets::ApplicationId)
                    .to(Applications::Table, Applications::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ApplicationSecrets {
    Table,
    ApplicationId,
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    Id,
}
