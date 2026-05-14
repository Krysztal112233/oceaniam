use sea_orm_migration::prelude::*;

const IDX_USERS_APP_ID_EMAIL: &str = "idx_users_app_id_email";
const IDX_USERS_APP_ID_PHONE: &str = "idx_users_app_id_phone";
const IDX_APPLICATIONS_TENANT_ID: &str = "idx_applications_tenant_id";
const IDX_APPLICATION_SECRETS_SECRET: &str = "idx_application_secrets_secret";
const IDX_REVOKED_JWTS_REVOKED_AT: &str = "idx_revoked_jwts_revoked_at";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_USERS_APP_ID_EMAIL)
                    .table(Users::Table)
                    .col(Users::ApplicationId)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_USERS_APP_ID_PHONE)
                    .table(Users::Table)
                    .col(Users::ApplicationId)
                    .col(Users::Phone)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_APPLICATIONS_TENANT_ID)
                    .table(Applications::Table)
                    .col(Applications::TenantId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_APPLICATION_SECRETS_SECRET)
                    .table(ApplicationSecrets::Table)
                    .col(ApplicationSecrets::Secret)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_REVOKED_JWTS_REVOKED_AT)
                    .table(RevokedJwts::Table)
                    .col((RevokedJwts::RevokedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_USERS_APP_ID_EMAIL)
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_USERS_APP_ID_PHONE)
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_APPLICATIONS_TENANT_ID)
                    .table(Applications::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_APPLICATION_SECRETS_SECRET)
                    .table(ApplicationSecrets::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_REVOKED_JWTS_REVOKED_AT)
                    .table(RevokedJwts::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    ApplicationId,
    Email,
    Phone,
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    TenantId,
}

#[derive(DeriveIden)]
enum ApplicationSecrets {
    Table,
    Secret,
}

#[derive(DeriveIden)]
enum RevokedJwts {
    Table,
    RevokedAt,
}
