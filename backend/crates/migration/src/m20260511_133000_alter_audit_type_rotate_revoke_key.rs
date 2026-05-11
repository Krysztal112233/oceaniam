use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_type(
                Type::alter()
                    .name(AuditTypeEnum::AuditType)
                    .add_value(AuditTypeEnum::RotateKey)
                    .if_not_exists(),
            )
            .await?;

        manager
            .alter_type(
                Type::alter()
                    .name(AuditTypeEnum::AuditType)
                    .add_value(AuditTypeEnum::RevokeKey)
                    .if_not_exists(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AuditTypeEnum {
    AuditType,

    RotateKey,
    RevokeKey,
}
