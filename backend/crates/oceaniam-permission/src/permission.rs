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

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn serde_round_trip() {
        for perm in [
            Permission::TenantRead,
            Permission::TenantCreate,
            Permission::ApplicationConfigurationPatch,
            Permission::SecretRead,
            Permission::KeyRotate,
            Permission::AdministratorCreate,
            Permission::ApplicationUserDelete,
            Permission::ApplicationTokenIssue,
            Permission::ApplicationChallengeRead,
        ] {
            let json = serde_json::to_string(&perm).unwrap();
            let deserialized: Permission = serde_json::from_str(&json).unwrap();
            assert_eq!(perm, deserialized);
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn strum_round_trip() {
        for perm in [
            Permission::TenantRead,
            Permission::TenantCreate,
            Permission::ApplicationConfigurationPatch,
            Permission::SecretRead,
            Permission::KeyRotate,
            Permission::AdministratorCreate,
            Permission::ApplicationUserDelete,
            Permission::ApplicationTokenIssue,
            Permission::ApplicationChallengeRead,
        ] {
            let s = perm.to_string();
            let parsed: Permission = s.parse().unwrap();
            assert_eq!(perm, parsed);
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn serialized_as_snake_case() {
        let json = serde_json::to_value(Permission::ApplicationUserRead).unwrap();
        assert_eq!(json, serde_json::json!("application_user_read"));
    }
}
