use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

use crate::permission::Permission;
use crate::sets;

// ── Platform roles ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PlatformRole {
    SuperAdmin,
    TenantAdmin,
    ReadonlyAdmin,
}

impl PlatformRole {
    pub fn permissions(&self) -> &'static HashSet<Permission> {
        match self {
            PlatformRole::SuperAdmin => &sets::PLATFORM_SUPER_ADMIN_PERMS,
            PlatformRole::TenantAdmin => &sets::PLATFORM_TENANT_ADMIN_PERMS,
            PlatformRole::ReadonlyAdmin => &sets::PLATFORM_READONLY_ADMIN_PERMS,
        }
    }
}

// ── Application roles ──

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AppRole {
    Owner,
    Admin,
    Member,
    Reader,
}

impl AppRole {
    pub fn permissions(&self) -> &'static HashSet<Permission> {
        match self {
            AppRole::Owner => &sets::APP_OWNER_PERMS,
            AppRole::Admin => &sets::APP_ADMIN_PERMS,
            AppRole::Member => &sets::APP_MEMBER_PERMS,
            AppRole::Reader => &sets::APP_READER_PERMS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn readonly_admin_has_no_write_permissions() {
        let perms = PlatformRole::ReadonlyAdmin.permissions();
        for p in [
            Permission::TenantCreate,
            Permission::TenantDelete,
            Permission::TenantPatch,
            Permission::ApplicationCreate,
            Permission::ApplicationDelete,
            Permission::ApplicationPatch,
            Permission::ApplicationConfigurationPatch,
            Permission::SecretCreate,
            Permission::SecretDelete,
            Permission::KeyRotate,
            Permission::KeyRevoke,
            Permission::AdministratorCreate,
            Permission::AdministratorPatch,
        ] {
            assert!(!perms.contains(&p), "ReadonlyAdmin should NOT have {p:?}");
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn readonly_admin_has_read_permissions() {
        let perms = PlatformRole::ReadonlyAdmin.permissions();
        for p in [
            Permission::TenantRead,
            Permission::ApplicationRead,
            Permission::ApplicationConfigurationRead,
            Permission::SecretRead,
            Permission::KeyRead,
            Permission::AdministratorRead,
        ] {
            assert!(perms.contains(&p), "ReadonlyAdmin should have {p:?}");
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn super_admin_has_all_platform_permissions() {
        let perms = PlatformRole::SuperAdmin.permissions();
        for p in [
            Permission::TenantCreate,
            Permission::TenantDelete,
            Permission::TenantPatch,
            Permission::TenantRead,
            Permission::ApplicationCreate,
            Permission::ApplicationDelete,
            Permission::ApplicationPatch,
            Permission::ApplicationRead,
            Permission::ApplicationConfigurationPatch,
            Permission::ApplicationConfigurationRead,
            Permission::SecretCreate,
            Permission::SecretDelete,
            Permission::SecretRead,
            Permission::KeyRead,
            Permission::KeyRotate,
            Permission::KeyRevoke,
            Permission::AdministratorCreate,
            Permission::AdministratorPatch,
            Permission::AdministratorRead,
        ] {
            assert!(perms.contains(&p), "SuperAdmin should have {p:?}");
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn tenant_admin_has_no_tenant_crud_and_no_admin_perms() {
        let perms = PlatformRole::TenantAdmin.permissions();
        for p in [
            Permission::TenantCreate,
            Permission::TenantDelete,
            Permission::AdministratorCreate,
            Permission::AdministratorPatch,
            Permission::AdministratorRead,
        ] {
            assert!(!perms.contains(&p), "TenantAdmin should NOT have {p:?}");
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn app_role_reader_is_subset_of_owner() {
        let reader = AppRole::Reader.permissions();
        let owner = AppRole::Owner.permissions();
        assert!(reader.is_subset(owner));
    }

    // NOTE: AI-generated test
    #[test]
    fn app_role_inheritance_chain() {
        assert!(
            AppRole::Reader
                .permissions()
                .is_subset(AppRole::Member.permissions())
        );
        assert!(
            AppRole::Member
                .permissions()
                .is_subset(AppRole::Admin.permissions())
        );
        assert!(
            AppRole::Admin
                .permissions()
                .is_subset(AppRole::Owner.permissions())
        );
    }
}
