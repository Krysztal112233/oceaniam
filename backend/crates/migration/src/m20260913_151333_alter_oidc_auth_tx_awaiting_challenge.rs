use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_type(
                Type::alter()
                    .name(
                        OidcAuthorizationTransactionStatusEnum::OidcAuthorizationTransactionStatus,
                    )
                    .add_value(OidcAuthorizationTransactionStatusEnum::AwaitingChallenge)
                    .if_not_exists(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OidcAuthorizationTransactions::Table)
                    .add_column(uuid_null(OidcAuthorizationTransactions::ChallengeId))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OidcAuthorizationTransactions::Table)
                    .drop_column(OidcAuthorizationTransactions::ChallengeId)
                    .to_owned(),
            )
            .await?;

        // PostgreSQL does not support removing an individual enum value safely. Keeping the
        // `awaiting_challenge` label is harmless after the corresponding application code is
        // rolled back.
        Ok(())
    }
}

#[derive(DeriveIden)]
enum OidcAuthorizationTransactions {
    Table,
    ChallengeId,
}

#[derive(DeriveIden)]
enum OidcAuthorizationTransactionStatusEnum {
    OidcAuthorizationTransactionStatus,
    AwaitingChallenge,
}
