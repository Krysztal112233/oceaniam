use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_CREDENTIAL_SUBJECT: &str = "fk_credential_id_subject_ref_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Credentials::Table)
                    .name(FK_CREDENTIAL_SUBJECT)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Credentials::Table)
                    .drop_column(Credentials::Value)
                    .add_column(string(Credentials::Phc))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Credentials::Table)
                    .drop_column(Credentials::Phc)
                    .add_column(json_binary(Credentials::Value))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Credentials::Table, Credentials::ID)
                    .to(Subjects::Table, Subjects::ID)
                    .name(FK_CREDENTIAL_SUBJECT)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Subjects {
    Table,

    ID,
}

#[derive(DeriveIden)]
enum Credentials {
    Table,

    ID,

    Phc,
    Value,
}
