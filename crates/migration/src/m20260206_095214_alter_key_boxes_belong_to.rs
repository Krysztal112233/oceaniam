use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_KEY_BOXES_APPLICATION: &str = "fk_key_boxes_id_application_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(FK_KEY_BOXES_APPLICATION)
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

        Ok(())
    }
}

#[derive(DeriveIden)]
enum KeyBoxes {
    Table,

    ApplicationID,
}

#[derive(DeriveIden)]
enum Applications {
    Table,

    ID,
}
