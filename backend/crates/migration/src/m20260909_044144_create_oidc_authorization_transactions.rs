use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

const UQ_APPLICATIONS_TENANT_ID_ID_OIDC_AUTH_TX: &str = "uq_applications_tenant_id_id_oidc_auth_tx";
const UQ_OIDC_CLIENTS_APPLICATION_ID_ID_AUTH_TX: &str = "uq_oidc_clients_application_id_id_auth_tx";
const IDX_OIDC_AUTH_TX_TENANT_APPLICATION: &str = "idx_oidc_auth_tx_tenant_application";
const IDX_OIDC_AUTH_TX_CLIENT_OWNER: &str = "idx_oidc_auth_tx_client_owner";
const IDX_OIDC_AUTH_TX_EXPIRES_AT: &str = "idx_oidc_auth_tx_expires_at";

const FK_OIDC_AUTH_TX_TENANT: &str = "fk_oidc_auth_tx_tenant";
const FK_OIDC_AUTH_TX_APPLICATION_OWNER: &str = "fk_oidc_auth_tx_application_owner";
const FK_OIDC_AUTH_TX_CLIENT_OWNER: &str = "fk_oidc_auth_tx_client_owner";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(
                        OidcAuthorizationTransactionStatusEnum::OidcAuthorizationTransactionStatus,
                    )
                    .values([
                        OidcAuthorizationTransactionStatusEnum::Pending,
                        OidcAuthorizationTransactionStatusEnum::Cancelled,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(OidcPkceMethodEnum::OidcPkceMethod)
                    .values([OidcPkceMethodEnum::S256])
                    .to_owned(),
            )
            .await?;

        // SeaORM codegen 2.0.1 discovers composite FK targets correctly from PostgreSQL
        // UNIQUE constraints, but not from standalone unique indexes. Create each index through
        // SeaQuery as required by repository policy, then attach it as the backing constraint.
        manager
            .create_index(
                Index::create()
                    .name(UQ_APPLICATIONS_TENANT_ID_ID_OIDC_AUTH_TX)
                    .table(Applications::Table)
                    .col(Applications::TenantId)
                    .col(Applications::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "ALTER TABLE applications ADD CONSTRAINT \
                 {UQ_APPLICATIONS_TENANT_ID_ID_OIDC_AUTH_TX} UNIQUE USING INDEX \
                 {UQ_APPLICATIONS_TENANT_ID_ID_OIDC_AUTH_TX}"
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_OIDC_CLIENTS_APPLICATION_ID_ID_AUTH_TX)
                    .table(OidcClients::Table)
                    .col(OidcClients::ApplicationId)
                    .col(OidcClients::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "ALTER TABLE oidc_clients ADD CONSTRAINT \
                 {UQ_OIDC_CLIENTS_APPLICATION_ID_ID_AUTH_TX} UNIQUE USING INDEX \
                 {UQ_OIDC_CLIENTS_APPLICATION_ID_ID_AUTH_TX}"
            ))
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OidcAuthorizationTransactions::Table)
                    .col(pk_uuid(OidcAuthorizationTransactions::Id))
                    .col(uuid(OidcAuthorizationTransactions::TenantId))
                    .col(uuid(OidcAuthorizationTransactions::ApplicationId))
                    .col(uuid(OidcAuthorizationTransactions::OidcClientId))
                    .col(text(OidcAuthorizationTransactions::Issuer))
                    .col(string_len(
                        OidcAuthorizationTransactions::RedirectUri,
                        2048,
                    ))
                    .col(
                        string_len(OidcAuthorizationTransactions::RequestedScope, 6)
                            .default("openid"),
                    )
                    .col(text(OidcAuthorizationTransactions::State))
                    .col(text_null(OidcAuthorizationTransactions::Nonce))
                    .col(string_len(
                        OidcAuthorizationTransactions::CodeChallenge,
                        43,
                    ))
                    .col(
                        enumeration(
                            OidcAuthorizationTransactions::CodeChallengeMethod,
                            OidcPkceMethodEnum::OidcPkceMethod,
                            [OidcPkceMethodEnum::S256],
                        )
                        .default(Expr::cust("'s256'::oidc_pkce_method")),
                    )
                    .col(binary(
                        OidcAuthorizationTransactions::BrowserBindingDigest,
                    ))
                    .col(binary(OidcAuthorizationTransactions::CsrfDigest))
                    .col(
                        enumeration(
                            OidcAuthorizationTransactions::Status,
                            OidcAuthorizationTransactionStatusEnum::OidcAuthorizationTransactionStatus,
                            [
                                OidcAuthorizationTransactionStatusEnum::Pending,
                                OidcAuthorizationTransactionStatusEnum::Cancelled,
                            ],
                        )
                        .default(Expr::cust(
                            "'pending'::oidc_authorization_transaction_status",
                        )),
                    )
                    .col(big_integer(OidcAuthorizationTransactions::Revision).default(0))
                    .col(timestamp_with_time_zone_null(
                        OidcAuthorizationTransactions::TerminalAt,
                    ))
                    .col(
                        timestamp_with_time_zone(OidcAuthorizationTransactions::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(OidcAuthorizationTransactions::ExpiresAt).default(
                            Expr::cust("CURRENT_TIMESTAMP + INTERVAL '10 minutes'"),
                        ),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_OIDC_AUTH_TX_TENANT)
                            .from(
                                OidcAuthorizationTransactions::Table,
                                OidcAuthorizationTransactions::TenantId,
                            )
                            .to(Tenants::Table, Tenants::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_OIDC_AUTH_TX_APPLICATION_OWNER)
                            .from_tbl(OidcAuthorizationTransactions::Table)
                            .from_col(OidcAuthorizationTransactions::TenantId)
                            .from_col(OidcAuthorizationTransactions::ApplicationId)
                            .to_tbl(Applications::Table)
                            .to_col(Applications::TenantId)
                            .to_col(Applications::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_OIDC_AUTH_TX_CLIENT_OWNER)
                            .from_tbl(OidcAuthorizationTransactions::Table)
                            .from_col(OidcAuthorizationTransactions::ApplicationId)
                            .from_col(OidcAuthorizationTransactions::OidcClientId)
                            .to_tbl(OidcClients::Table)
                            .to_col(OidcClients::ApplicationId)
                            .to_col(OidcClients::Id)
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_OIDC_AUTH_TX_TENANT_APPLICATION)
                    .table(OidcAuthorizationTransactions::Table)
                    .col(OidcAuthorizationTransactions::TenantId)
                    .col(OidcAuthorizationTransactions::ApplicationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_OIDC_AUTH_TX_CLIENT_OWNER)
                    .table(OidcAuthorizationTransactions::Table)
                    .col(OidcAuthorizationTransactions::ApplicationId)
                    .col(OidcAuthorizationTransactions::OidcClientId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_OIDC_AUTH_TX_EXPIRES_AT)
                    .table(OidcAuthorizationTransactions::Table)
                    .col(OidcAuthorizationTransactions::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(OidcAuthorizationTransactions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(&format!(
                "ALTER TABLE oidc_clients DROP CONSTRAINT \
                 {UQ_OIDC_CLIENTS_APPLICATION_ID_ID_AUTH_TX}"
            ))
            .await?;

        manager
            .get_connection()
            .execute_unprepared(&format!(
                "ALTER TABLE applications DROP CONSTRAINT \
                 {UQ_APPLICATIONS_TENANT_ID_ID_OIDC_AUTH_TX}"
            ))
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(OidcPkceMethodEnum::OidcPkceMethod)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(
                        OidcAuthorizationTransactionStatusEnum::OidcAuthorizationTransactionStatus,
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum OidcAuthorizationTransactions {
    Table,
    Id,
    TenantId,
    ApplicationId,
    OidcClientId,
    Issuer,
    RedirectUri,
    RequestedScope,
    State,
    Nonce,
    CodeChallenge,
    CodeChallengeMethod,
    BrowserBindingDigest,
    CsrfDigest,
    Status,
    Revision,
    TerminalAt,
    CreatedAt,
    ExpiresAt,
}

#[derive(DeriveIden)]
enum OidcAuthorizationTransactionStatusEnum {
    OidcAuthorizationTransactionStatus,
    Pending,
    Cancelled,
}

#[derive(DeriveIden)]
enum OidcPkceMethodEnum {
    OidcPkceMethod,
    S256,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    Id,
    TenantId,
}

#[derive(DeriveIden)]
enum OidcClients {
    Table,
    Id,
    ApplicationId,
}
