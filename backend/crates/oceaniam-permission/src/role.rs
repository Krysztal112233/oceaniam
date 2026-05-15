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
            PlatformRole::SuperAdmin => &*sets::PLATFORM_SUPER_ADMIN_PERMS,
            PlatformRole::TenantAdmin => &*sets::PLATFORM_TENANT_ADMIN_PERMS,
            PlatformRole::ReadonlyAdmin => &*sets::PLATFORM_READONLY_ADMIN_PERMS,
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
            AppRole::Owner => &*sets::APP_OWNER_PERMS,
            AppRole::Admin => &*sets::APP_ADMIN_PERMS,
            AppRole::Member => &*sets::APP_MEMBER_PERMS,
            AppRole::Reader => &*sets::APP_READER_PERMS,
        }
    }
}
