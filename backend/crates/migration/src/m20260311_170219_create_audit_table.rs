use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AuditTypeEnum::AuditType)
                    .values([
                        AuditTypeEnum::SignJWT,
                        AuditTypeEnum::RevokeJWT,
                        AuditTypeEnum::CreateApplication,
                        AuditTypeEnum::DeleteApplication,
                        AuditTypeEnum::CreateTenants,
                        AuditTypeEnum::DeleteTenants,
                        AuditTypeEnum::CreateApplicationUser,
                        AuditTypeEnum::DeleteApplicationUser,
                        AuditTypeEnum::CreateApplicationSecret,
                        AuditTypeEnum::DeleteApplicationSecret,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Audits::Table)
                    .col(pk_uuid(Audits::Id))
                    .col(enumeration(
                        Audits::AuditType,
                        AuditTypeEnum::AuditType,
                        [
                            AuditTypeEnum::SignJWT,
                            AuditTypeEnum::RevokeJWT,
                            AuditTypeEnum::CreateApplication,
                            AuditTypeEnum::DeleteApplication,
                            AuditTypeEnum::CreateTenants,
                            AuditTypeEnum::DeleteTenants,
                            AuditTypeEnum::CreateApplicationUser,
                            AuditTypeEnum::DeleteApplicationUser,
                            AuditTypeEnum::CreateApplicationSecret,
                            AuditTypeEnum::DeleteApplicationSecret,
                        ],
                    ))
                    .col(
                        json_binary(Audits::Payload)
                            .not_null()
                            .default(Expr::cust("'{}'::jsonb")),
                    )
                    .col(
                        timestamp_with_time_zone(Audits::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Audits::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AuditTypeEnum::AuditType).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Audits {
    Table,
    Id,
    AuditType,
    Payload,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AuditTypeEnum {
    AuditType,

    SignJWT,
    RevokeJWT,

    CreateApplication,
    DeleteApplication,

    CreateTenants,
    DeleteTenants,

    CreateApplicationUser,
    DeleteApplicationUser,

    CreateApplicationSecret,
    DeleteApplicationSecret,
}
