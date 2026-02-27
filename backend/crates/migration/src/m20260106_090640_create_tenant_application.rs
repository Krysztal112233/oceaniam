use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_APPLICATION_TENANTS: &str = "fk_applications_id_tenants_id";
const FK_USER_APPLICATION: &str = "fk_user_id_application_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tenants::Table)
                    .col(pk_uuid(Tenants::ID))
                    .col(string(Tenants::Comment))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Applications::Table)
                    .col(pk_uuid(Applications::ID))
                    .col(string(Applications::Comment))
                    .col(uuid(Applications::TenantsID))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(uuid(Users::ApplicationID))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_USER_APPLICATION)
                    .from(Users::Table, Users::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_APPLICATION_TENANTS)
                    .from(Applications::Table, Applications::TenantsID)
                    .to(Tenants::Table, Tenants::ID)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_USER_APPLICATION)
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_APPLICATION_TENANTS)
                    .table(Applications::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::ApplicationID)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Applications::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tenants::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Tenants {
    Table,

    ID,
    Comment,
}

#[derive(DeriveIden)]
enum Applications {
    Table,

    TenantsID,
    ID,
    Comment,
}

#[derive(DeriveIden)]
enum Users {
    Table,

    ApplicationID,
}
