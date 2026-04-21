use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Challenges::Table)
                    .if_not_exists()
                    .col(pk_uuid(Challenges::Id))
                    .col(string(Challenges::Token))
                    .col(timestamp_with_time_zone(Challenges::ExpiresAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Challenges::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Challenges {
    Table,
    Id,
    Token,
    ExpiresAt,
}
