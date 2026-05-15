use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    // ── Platform: Tenants ──
    TenantCreate,
    TenantDelete,
    TenantPatch,
    TenantRead,

    // ── Platform: Applications ──
    ApplicationCreate,
    ApplicationDelete,
    ApplicationPatch,
    ApplicationRead,

    // ── Platform: Application Configuration ──
    ApplicationConfigurationPatch,
    ApplicationConfigurationRead,

    // ── Platform: Application Secrets ──
    SecretCreate,
    SecretDelete,
    SecretRead,

    // ── Platform: Application Keys ──
    KeyRead,
    KeyRevoke,
    KeyRotate,

    // ── Platform: Administrators ──
    AdministratorCreate,
    AdministratorPatch,
    AdministratorRead,

    // ── Application-level: Users ──
    ApplicationUserDelete,
    ApplicationUserInvite,
    ApplicationUserPatch,
    ApplicationUserRead,

    // ── Application-level: Tokens ──
    ApplicationTokenIssue,
    ApplicationTokenRevoke,

    // ── Application-level: Challenges ──
    ApplicationChallengeRead,
}
