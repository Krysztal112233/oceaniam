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
                    .add_value(OidcAuthorizationTransactionStatusEnum::Authenticated)
                    .if_not_exists(),
            )
            .await?;

        manager
            .alter_type(
                Type::alter()
                    .name(AuditTypeEnum::AuditType)
                    .add_value(AuditTypeEnum::OidcAuthenticate)
                    .if_not_exists(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OidcAuthorizationTransactions::Table)
                    .add_column(uuid_null(OidcAuthorizationTransactions::SubjectId))
                    .add_column(timestamp_with_time_zone_null(
                        OidcAuthorizationTransactions::AuthenticatedAt,
                    ))
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260913_094443_alter_oidc_auth_tx_authenticated/up.sql"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260913_094443_alter_oidc_auth_tx_authenticated/down.sql"
            ))
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OidcAuthorizationTransactions::Table)
                    .drop_column(OidcAuthorizationTransactions::AuthenticatedAt)
                    .drop_column(OidcAuthorizationTransactions::SubjectId)
                    .to_owned(),
            )
            .await?;

        // PostgreSQL does not support removing an individual enum value safely. Keeping the
        // `authenticated` and `oidc_authenticate` labels is harmless after the corresponding
        // application code is rolled back.
        Ok(())
    }
}

#[derive(DeriveIden)]
enum OidcAuthorizationTransactions {
    Table,
    SubjectId,
    AuthenticatedAt,
}

#[derive(DeriveIden)]
enum OidcAuthorizationTransactionStatusEnum {
    OidcAuthorizationTransactionStatus,
    Authenticated,
}

#[derive(DeriveIden)]
enum AuditTypeEnum {
    AuditType,
    OidcAuthenticate,
}
