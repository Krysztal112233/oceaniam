use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Credentials::Table)
                    .add_column_if_not_exists(string_null(Credentials::Totp).comment("This field is optional. Its value is produced by serializing the TOTP struct provided by the totp_rs crate to JSON and encrypting it with XChaCha20Poly1305. It must be decrypted before each use. If decryption fails, the TOTP function cannot continue to be used and must be regenerated."))
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
                    .drop_column(Credentials::Totp)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Credentials {
    Table,

    Totp,
}
