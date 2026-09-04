use sea_orm_migration::{prelude::*, schema::*};

const FK_OIDC_CLIENTS_APPLICATION: &str = "fk_oidc_clients_application_id";
const FK_OIDC_CLIENT_REDIRECT_URIS_CLIENT: &str = "fk_oidc_client_redirect_uris_oidc_client_id";
const IDX_OIDC_CLIENTS_APPLICATION_ID: &str = "idx_oidc_clients_application_id";
const UQ_OIDC_CLIENTS_CLIENT_ID: &str = "uq_oidc_clients_client_id";
const UQ_OIDC_CLIENT_REDIRECT_URIS_CLIENT_URI: &str = "uq_oidc_client_redirect_uris_client_uri";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OidcClients::Table)
                    .col(pk_uuid(OidcClients::Id))
                    .col(uuid(OidcClients::ApplicationId).not_null())
                    .col(
                        ColumnDef::new(OidcClients::ClientId)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(ColumnDef::new(OidcClients::Name).string_len(128).not_null())
                    .col(
                        ColumnDef::new(OidcClients::ClientType)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OidcClients::ApplicationType)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(OidcClients::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .check(Expr::col(OidcClients::ClientType).eq("public"))
                    .check(Expr::col(OidcClients::ApplicationType).eq("web"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_OIDC_CLIENTS_APPLICATION)
                    .from(OidcClients::Table, OidcClients::ApplicationId)
                    .to(Applications::Table, Applications::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_OIDC_CLIENTS_APPLICATION_ID)
                    .table(OidcClients::Table)
                    .col(OidcClients::ApplicationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_OIDC_CLIENTS_CLIENT_ID)
                    .table(OidcClients::Table)
                    .col(OidcClients::ClientId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OidcClientRedirectUris::Table)
                    .col(pk_uuid(OidcClientRedirectUris::Id))
                    .col(uuid(OidcClientRedirectUris::OidcClientId).not_null())
                    .col(
                        ColumnDef::new(OidcClientRedirectUris::RedirectUri)
                            .string_len(2048)
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(OidcClientRedirectUris::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_OIDC_CLIENT_REDIRECT_URIS_CLIENT)
                    .from(
                        OidcClientRedirectUris::Table,
                        OidcClientRedirectUris::OidcClientId,
                    )
                    .to(OidcClients::Table, OidcClients::Id)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_OIDC_CLIENT_REDIRECT_URIS_CLIENT_URI)
                    .table(OidcClientRedirectUris::Table)
                    .col(OidcClientRedirectUris::OidcClientId)
                    .col(OidcClientRedirectUris::RedirectUri)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(OidcClientRedirectUris::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(OidcClients::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum OidcClients {
    Table,
    Id,
    ApplicationId,
    ClientId,
    Name,
    ClientType,
    ApplicationType,
    CreatedAt,
}

#[derive(DeriveIden)]
enum OidcClientRedirectUris {
    Table,
    Id,
    OidcClientId,
    RedirectUri,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    Id,
}
