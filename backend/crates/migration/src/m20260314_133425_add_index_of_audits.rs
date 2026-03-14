use sea_orm_migration::prelude::*;

const IDX_AUDITS_CREATED_AT: &str = "idx_audits_created_at";
const IDX_AUDITS_AUDIT_TYPE_CREATED_AT: &str = "idx_audits_audit_type_created_at";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(IDX_AUDITS_CREATED_AT)
                    .table(Audits::Table)
                    .col((Audits::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_AUDITS_AUDIT_TYPE_CREATED_AT)
                    .table(Audits::Table)
                    .col(Audits::AuditType)
                    .col((Audits::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260314_133425_add_index_of_audits/up.sql"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_AUDITS_AUDIT_TYPE_CREATED_AT)
                    .table(Audits::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_AUDITS_CREATED_AT)
                    .table(Audits::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260314_133425_add_index_of_audits/down.sql"
            ))
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Audits {
    Table,
    AuditType,
    CreatedAt,
}
