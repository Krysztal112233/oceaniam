use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_KEY_BOXES_APPLICATION: &str = "fk_key_boxes_id_application_id";
const FK_KEY_BOXES_TENANT: &str = "fk_key_boxes_id_tenant_id";
const IDX_KEY_BOXES_APPLICATION_ID: &str = "idx_key_boxes_application_id";
const IDX_KEY_BOXES_TENANT_ID: &str = "idx_key_boxes_tenant_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .truncate_table(Table::truncate().table(KeyBoxes::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(KeyBoxes::Table)
                    .name(FK_KEY_BOXES_APPLICATION)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_KEY_BOXES_APPLICATION_ID)
                    .table(KeyBoxes::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(KeyBoxes::Table)
                    .drop_column(KeyBoxes::ApplicationID)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(KeyBoxes::Table)
                    .add_column(uuid(KeyBoxes::TenantID))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_KEY_BOXES_TENANT)
                    .from(KeyBoxes::Table, KeyBoxes::TenantID)
                    .to(Tenants::Table, Tenants::ID)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_KEY_BOXES_TENANT_ID)
                    .table(KeyBoxes::Table)
                    .col(KeyBoxes::TenantID)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .truncate_table(Table::truncate().table(KeyBoxes::Table).to_owned())
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(KeyBoxes::Table)
                    .name(FK_KEY_BOXES_TENANT)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_KEY_BOXES_TENANT_ID)
                    .table(KeyBoxes::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(KeyBoxes::Table)
                    .drop_column(KeyBoxes::TenantID)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(KeyBoxes::Table)
                    .add_column(uuid(KeyBoxes::ApplicationID))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_KEY_BOXES_APPLICATION)
                    .from(KeyBoxes::Table, KeyBoxes::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
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
}

#[derive(DeriveIden)]
enum KeyBoxes {
    Table,

    ApplicationID,
    TenantID,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,

    ID,
}

#[derive(DeriveIden)]
enum Applications {
    Table,

    ID,
}
