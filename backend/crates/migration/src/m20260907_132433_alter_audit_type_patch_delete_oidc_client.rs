use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260907_132433_alter_audit_type_patch_delete_oidc_client"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_type(
                Type::alter()
                    .name(AuditTypeEnum::AuditType)
                    .add_value(AuditTypeEnum::PatchOidcClient)
                    .if_not_exists(),
            )
            .await?;

        manager
            .alter_type(
                Type::alter()
                    .name(AuditTypeEnum::AuditType)
                    .add_value(AuditTypeEnum::DeleteOidcClient)
                    .if_not_exists(),
            )
            .await?;

        Ok(())
    }

    // PostgreSQL cannot safely remove individual enum values during rollback.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AuditTypeEnum {
    AuditType,
    PatchOidcClient,
    DeleteOidcClient,
}
