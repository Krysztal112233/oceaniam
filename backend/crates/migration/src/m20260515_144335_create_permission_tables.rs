use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_APPLICATION_ROLES_APPLICATION: &str = "fk_application_roles_application_id";
const FK_SUBJECTS_APPLICATION_ROLE: &str = "fk_subjects_application_role_id";
const FK_ADMINISTRATOR_TENANTS_ADMINISTRATOR: &str = "fk_administrator_tenants_administrator_id";
const FK_ADMINISTRATOR_TENANTS_TENANT: &str = "fk_administrator_tenants_tenant_id";

const IDX_APPLICATION_ROLES_APPLICATION_ID: &str = "idx_application_roles_application_id";
const IDX_SUBJECTS_APPLICATION_ROLE_ID: &str = "idx_subjects_application_role_id";
const IDX_ADMINISTRATOR_TENANTS_ADMINISTRATOR_ID: &str =
    "idx_administrator_tenants_administrator_id";
const IDX_ADMINISTRATOR_TENANTS_TENANT_ID: &str = "idx_administrator_tenants_tenant_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. Add role column to administrators ──
        manager
            .alter_table(
                Table::alter()
                    .table(Administrators::Table)
                    .add_column_if_not_exists(string_null(Administrators::Role))
                    .to_owned(),
            )
            .await?;

        // ── 2. administrator_tenants ──
        manager
            .create_table(
                Table::create()
                    .table(AdministratorTenants::Table)
                    .if_not_exists()
                    .col(pk_uuid(AdministratorTenants::Id))
                    .col(uuid(AdministratorTenants::AdministratorId).not_null())
                    .col(uuid(AdministratorTenants::TenantId).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_ADMINISTRATOR_TENANTS_ADMINISTRATOR)
                    .from(
                        AdministratorTenants::Table,
                        AdministratorTenants::AdministratorId,
                    )
                    .to(Administrators::Table, Administrators::ID)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_ADMINISTRATOR_TENANTS_TENANT)
                    .from(AdministratorTenants::Table, AdministratorTenants::TenantId)
                    .to(Tenants::Table, Tenants::ID)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_ADMINISTRATOR_TENANTS_ADMINISTRATOR_ID)
                    .table(AdministratorTenants::Table)
                    .col(AdministratorTenants::AdministratorId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_ADMINISTRATOR_TENANTS_TENANT_ID)
                    .table(AdministratorTenants::Table)
                    .col(AdministratorTenants::TenantId)
                    .to_owned(),
            )
            .await?;

        // ── 3. application_roles ──
        manager
            .create_table(
                Table::create()
                    .table(ApplicationRoles::Table)
                    .if_not_exists()
                    .col(pk_uuid(ApplicationRoles::Id))
                    .col(uuid(ApplicationRoles::ApplicationId).not_null())
                    .col(string(ApplicationRoles::Name).not_null())
                    .col(boolean(ApplicationRoles::IsSystem).not_null().default(true))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_APPLICATION_ROLES_APPLICATION_ID)
                    .table(ApplicationRoles::Table)
                    .col(ApplicationRoles::ApplicationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_APPLICATION_ROLES_APPLICATION)
                    .from(ApplicationRoles::Table, ApplicationRoles::ApplicationId)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // ── 4. Add application_role_id to subjects ──
        manager
            .alter_table(
                Table::alter()
                    .table(Subjects::Table)
                    .add_column_if_not_exists(uuid_null(Subjects::ApplicationRoleId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_SUBJECTS_APPLICATION_ROLE)
                    .from(Subjects::Table, Subjects::ApplicationRoleId)
                    .to(ApplicationRoles::Table, ApplicationRoles::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_SUBJECTS_APPLICATION_ROLE_ID)
                    .table(Subjects::Table)
                    .col(Subjects::ApplicationRoleId)
                    .to_owned(),
            )
            .await?;

        // ── Seed data ──
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260515_144335_create_permission_tables/up.sql"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── Revert seed data ──
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260515_144335_create_permission_tables/down.sql"
            ))
            .await?;

        // ── Drop FK, then tables, then columns ──
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Subjects::Table)
                    .name(FK_SUBJECTS_APPLICATION_ROLE)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Subjects::Table)
                    .drop_column(Subjects::ApplicationRoleId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ApplicationRoles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AdministratorTenants::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Administrators::Table)
                    .drop_column(Administrators::Role)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// ── Table idens ──

#[derive(DeriveIden)]
enum Administrators {
    Table,
    ID,
    Role,
}

#[derive(DeriveIden)]
enum Tenants {
    Table,
    ID,
}

#[derive(DeriveIden)]
enum AdministratorTenants {
    Table,
    Id,
    AdministratorId,
    TenantId,
}

#[derive(DeriveIden)]
enum Subjects {
    Table,
    ApplicationRoleId,
}

#[derive(DeriveIden)]
enum ApplicationRoles {
    Table,
    Id,
    ApplicationId,
    Name,
    IsSystem,
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    ID,
}
