use sea_orm_migration::{prelude::*, schema::*};

const UQ_OIDC_AUTH_CODE_DIGEST: &str = "uq_oidc_auth_code_digest";
const IDX_OIDC_AUTH_CODE_EXPIRES_AT: &str = "idx_oidc_auth_code_expires_at";

const FK_OIDC_AUTH_CODE_TRANSACTION: &str = "fk_oidc_auth_code_transaction";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OidcAuthorizationCodes::Table)
                    .col(pk_uuid(OidcAuthorizationCodes::Id))
                    .col(uuid(OidcAuthorizationCodes::TransactionId))
                    .col(binary(OidcAuthorizationCodes::CodeDigest))
                    .col(string_len(OidcAuthorizationCodes::RedirectUri, 2048))
                    .col(uuid(OidcAuthorizationCodes::SubjectId))
                    .col(timestamp_with_time_zone(OidcAuthorizationCodes::ExpiresAt))
                    .col(timestamp_with_time_zone_null(
                        OidcAuthorizationCodes::RedeemedAt,
                    ))
                    .col(timestamp_with_time_zone(OidcAuthorizationCodes::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_OIDC_AUTH_CODE_TRANSACTION)
                            .from(
                                OidcAuthorizationCodes::Table,
                                OidcAuthorizationCodes::TransactionId,
                            )
                            .to(
                                OidcAuthorizationTransactions::Table,
                                OidcAuthorizationTransactions::Id,
                            )
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // The digest is the code's sole lookup key (Rule 2: a UNIQUE key identifies the row
        // exactly like a primary key), so `consume_authorization_code` filters on it.
        manager
            .create_index(
                Index::create()
                    .name(UQ_OIDC_AUTH_CODE_DIGEST)
                    .table(OidcAuthorizationCodes::Table)
                    .col(OidcAuthorizationCodes::CodeDigest)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_OIDC_AUTH_CODE_EXPIRES_AT)
                    .table(OidcAuthorizationCodes::Table)
                    .col(OidcAuthorizationCodes::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(OidcAuthorizationCodes::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum OidcAuthorizationCodes {
    Table,
    Id,
    TransactionId,
    CodeDigest,
    RedirectUri,
    SubjectId,
    ExpiresAt,
    RedeemedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum OidcAuthorizationTransactions {
    Table,
    Id,
}
