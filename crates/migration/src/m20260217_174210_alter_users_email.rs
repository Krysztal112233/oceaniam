use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // add email column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string_null(Users::Email))
                    .to_owned(),
            )
            .await?;

        // add phone column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(string_null(Users::Phone))
                    .to_owned(),
            )
            .await?;

        // add check constraint: email or phone must be not null
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE users ADD CONSTRAINT chk_users_email_or_phone_not_null CHECK (email IS NOT NULL OR phone IS NOT NULL)",
            )
            .await?;

        // drop name column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // add name column with uuid string as default
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::Name)
                            .string()
                            .not_null()
                            .default("id::text"),
                    )
                    .to_owned(),
            )
            .await?;

        // update name column with uuid string for existing rows
        manager
            .get_connection()
            .execute_unprepared("UPDATE users SET name = id::text")
            .await?;

        // drop check constraint
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE users DROP CONSTRAINT chk_users_email_or_phone_not_null",
            )
            .await?;

        // drop email column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Email)
                    .to_owned(),
            )
            .await?;

        // drop phone column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::Phone)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,

    Name,
    Email,
    Phone,
}
