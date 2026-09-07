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
                    .add_value(AuditTypeEnum::CreateOidcClient)
                    .if_not_exists(),
            )
            .await?;

        Ok(())
    }

    // PostgreSQL does not support removing an individual enum value safely. Keeping the value is
    // harmless after the corresponding application code is rolled back.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AuditTypeEnum {
    AuditType,
    CreateOidcClient,
}
