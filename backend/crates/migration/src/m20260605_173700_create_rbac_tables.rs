use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_SUBJECTS_APPLICATION_ROLE: &str = "fk_subjects_application_role_id";
const FK_ROLE_PERMISSIONS_ROLE_ID: &str = "fk_role_permissions_role_id";
const FK_SUBJECT_ROLES_SUBJECT_ID: &str = "fk_subject_roles_subject_id";
const FK_SUBJECT_ROLES_ROLE_ID: &str = "fk_subject_roles_role_id";

const IDX_SUBJECTS_APPLICATION_ROLE_ID: &str = "idx_subjects_application_role_id";
const IDX_ROLE_PERMISSIONS_ROLE_ID: &str = "idx_role_permissions_role_id";
const IDX_SUBJECT_ROLES_SUBJECT_ID: &str = "idx_subject_roles_subject_id";
const IDX_SUBJECT_ROLES_ROLE_ID: &str = "idx_subject_roles_role_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. Drop FK and index on subjects.application_role_id ──
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Subjects::Table)
                    .name(FK_SUBJECTS_APPLICATION_ROLE)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_SUBJECTS_APPLICATION_ROLE_ID)
                    .table(Subjects::Table)
                    .to_owned(),
            )
            .await?;

        // ── 2. Create role_permissions (composite PK via raw SQL) ──
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS role_permissions (
                    role_id    UUID    NOT NULL,
                    permission VARCHAR NOT NULL,
                    PRIMARY KEY (role_id, permission)
                )",
            )
            .await?;

        // ── 3. Create subject_roles (composite PK via raw SQL) ──
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS subject_roles (
                    subject_id UUID NOT NULL,
                    role_id    UUID NOT NULL,
                    PRIMARY KEY (subject_id, role_id)
                )",
            )
            .await?;

        // ── 4. Foreign keys ──
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_ROLE_PERMISSIONS_ROLE_ID)
                    .from(RolePermissions::Table, RolePermissions::RoleId)
                    .to(ApplicationRoles::Table, ApplicationRoles::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_SUBJECT_ROLES_SUBJECT_ID)
                    .from(SubjectRoles::Table, SubjectRoles::SubjectId)
                    .to(Subjects::Table, Subjects::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_SUBJECT_ROLES_ROLE_ID)
                    .from(SubjectRoles::Table, SubjectRoles::RoleId)
                    .to(ApplicationRoles::Table, ApplicationRoles::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // ── 5. Indexes ──
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_ROLE_PERMISSIONS_ROLE_ID)
                    .table(RolePermissions::Table)
                    .col(RolePermissions::RoleId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_SUBJECT_ROLES_SUBJECT_ID)
                    .table(SubjectRoles::Table)
                    .col(SubjectRoles::SubjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_SUBJECT_ROLES_ROLE_ID)
                    .table(SubjectRoles::Table)
                    .col(SubjectRoles::RoleId)
                    .to_owned(),
            )
            .await?;

        // ── 6. Seed role_permissions ──
        manager
            .get_connection()
            .execute_unprepared(include_str!("./m20260605_173700_create_rbac_tables/up.sql"))
            .await?;

        // ── 7. Migrate existing data from subjects → subject_roles ──
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO subject_roles (subject_id, role_id)
                 SELECT id, application_role_id
                 FROM subjects
                 WHERE application_role_id IS NOT NULL
                 ON CONFLICT DO NOTHING",
            )
            .await?;

        // ── 8. Drop application_role_id from subjects ──
        manager
            .alter_table(
                Table::alter()
                    .table(Subjects::Table)
                    .drop_column(Subjects::ApplicationRoleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. Re-add application_role_id to subjects ──
        manager
            .alter_table(
                Table::alter()
                    .table(Subjects::Table)
                    .add_column_if_not_exists(uuid_null(Subjects::ApplicationRoleId))
                    .to_owned(),
            )
            .await?;

        // ── 2. Restore data from subject_roles → subjects.application_role_id ──
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE subjects s
                 SET application_role_id = sr.role_id
                 FROM subject_roles sr
                 WHERE sr.subject_id = s.id",
            )
            .await?;

        // ── 3. Re-create FK and index on subjects.application_role_id ──
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

        // ── 4. Delete seed data ──
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260605_173700_create_rbac_tables/down.sql"
            ))
            .await?;

        // ── 5. Drop FKs on subject_roles and role_permissions ──
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(SubjectRoles::Table)
                    .name(FK_SUBJECT_ROLES_SUBJECT_ID)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(SubjectRoles::Table)
                    .name(FK_SUBJECT_ROLES_ROLE_ID)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(RolePermissions::Table)
                    .name(FK_ROLE_PERMISSIONS_ROLE_ID)
                    .to_owned(),
            )
            .await?;

        // ── 6. Drop indexes ──
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_ROLE_PERMISSIONS_ROLE_ID)
                    .table(RolePermissions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_SUBJECT_ROLES_SUBJECT_ID)
                    .table(SubjectRoles::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_SUBJECT_ROLES_ROLE_ID)
                    .table(SubjectRoles::Table)
                    .to_owned(),
            )
            .await?;

        // ── 7. Drop tables ──
        manager
            .drop_table(Table::drop().table(SubjectRoles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RolePermissions::Table).to_owned())
            .await?;

        Ok(())
    }
}

// ── Table idens ──

#[derive(DeriveIden)]
enum Subjects {
    Table,
    Id,
    ApplicationRoleId,
}

#[derive(DeriveIden)]
enum ApplicationRoles {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum RolePermissions {
    Table,
    RoleId,
    Permission,
}

#[derive(DeriveIden)]
enum SubjectRoles {
    Table,
    SubjectId,
    RoleId,
}
