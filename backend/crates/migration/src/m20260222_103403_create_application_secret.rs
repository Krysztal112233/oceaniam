use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_APPLICATION_SECRET_APPLICATION: &str = "fk_application_secret_application_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApplicationSecrets::Table)
                    .col(pk_uuid(ApplicationSecrets::Id))
                    .col(uuid(ApplicationSecrets::ApplicationId))
                    .col(string(ApplicationSecrets::Secret))
                    .col(timestamp_with_time_zone(ApplicationSecrets::CreatedAt))
                    .col(timestamp_with_time_zone_null(ApplicationSecrets::RevokedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_APPLICATION_SECRET_APPLICATION)
                    .from(ApplicationSecrets::Table, ApplicationSecrets::ApplicationId)
                    .to(Applications::Table, Applications::Id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_APPLICATION_SECRET_APPLICATION)
                    .table(ApplicationSecrets::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ApplicationSecrets::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ApplicationSecrets {
    Table,

    Id,

    ApplicationId,
    Secret,

    CreatedAt,
    RevokedAt,
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    Id,
}
