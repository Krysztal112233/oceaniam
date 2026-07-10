use chrono::{DateTime, FixedOffset};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::patch::PatchValue;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateApplicationRequest {
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateApplicationResponse {
    pub tenant_id: String,
    pub application_id: String,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TokenConfigurationVO {
    pub issuer: String,
    pub audience: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Argon2Configuration {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PasswordConfigurationVO {
    pub argon2: Argon2Configuration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuthConfigurationVO {
    pub token: TokenConfigurationVO,
    pub password: PasswordConfigurationVO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RegistrationConfigurationVO {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApplicationConfigurationVO {
    pub auth: AuthConfigurationVO,
    pub registration: RegistrationConfigurationVO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
pub struct PatchTokenConfigurationVO {
    pub issuer: Option<String>,
    pub audience: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
pub struct PatchAuthConfigurationVO {
    pub token: Option<PatchTokenConfigurationVO>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
pub struct PatchRegistrationConfigurationVO {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
pub struct PatchApplicationConfigurationRequest {
    pub auth: Option<PatchAuthConfigurationVO>,
    pub registration: Option<PatchRegistrationConfigurationVO>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PatchApplicationRequest {
    #[serde(default, skip_serializing_if = "PatchValue::is_missing")]
    #[schema(value_type = Option<String>)]
    pub comment: PatchValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GetApplicationConfigurationResponse {
    pub configuration: ApplicationConfigurationVO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApplicationDetailVO {
    pub id: String,
    pub comment: Option<String>,
    pub tenant_id: String,
    pub configuration: ApplicationConfigurationVO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApplicationVO {
    pub id: String,
    pub comment: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApplicationChallengeVO {
    pub id: String,
    pub application_id: String,
    pub subject_id: String,
    pub factor_type: String,
    pub purpose: String,
    pub status: String,
    pub attempt_count: i32,
    pub remaining_attempts: i32,
    pub expires_at: DateTime<FixedOffset>,
    pub consumed_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
}

/// VO for creating a new application user
///
/// Either `phone` or `email` must be provided (mutually exclusive)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
pub struct CreateApplicationUserRequest {
    /// User email address (optional, but either phone or email must be provided)
    #[garde(email)]
    pub email: Option<String>,
    /// User phone number (optional, but either phone or email must be provided)
    #[garde(phone_number)]
    pub phone: Option<String>,

    /// User nickname (optional, if not provided, a random name will be generated)
    #[garde(length(min = 4))]
    pub nickname: Option<String>,

    #[garde(skip)]
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
pub struct PatchApplicationUserRequest {
    /// New nickname (optional; when present must be at least 4 characters)
    #[garde(length(min = 4))]
    pub nickname: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
pub struct PatchApplicationUserCredentialsRequest {
    #[garde(length(min = 12))]
    pub password: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationUsersSortOrder {
    Asc,

    #[default]
    Desc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
#[serde(default)]
pub struct ApplicationUsersListQuery {
    #[garde(skip)]
    pub page: u64,

    #[garde(range(min = 0, max = 1024))]
    pub per_page: u64,

    #[garde(skip)]
    pub sort_order: ApplicationUsersSortOrder,
}

impl Default for ApplicationUsersListQuery {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: 30,
            sort_order: ApplicationUsersSortOrder::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
#[serde(default)]
pub struct SearchApplicationUsersQuery {
    #[garde(skip)]
    pub page: u64,

    #[garde(range(min = 0, max = 1024))]
    pub per_page: u64,

    #[garde(skip)]
    pub sort_order: ApplicationUsersSortOrder,

    #[garde(custom(forbid_search_wildcards))]
    pub by_nickname: Option<String>,
    #[garde(custom(forbid_search_wildcards))]
    pub by_email: Option<String>,
    #[garde(custom(forbid_search_wildcards))]
    pub by_phone: Option<String>,
    #[garde(skip)]
    pub by_id: Option<String>,
}

impl Default for SearchApplicationUsersQuery {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: 30,
            sort_order: ApplicationUsersSortOrder::default(),
            by_nickname: None,
            by_email: None,
            by_phone: None,
            by_id: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApplicationUserVO {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SecretVO {
    pub id: String,
    pub secret: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
    pub application_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApplicationKeyVO {
    pub key_id: String,
    pub algorithm: String,
    pub status: String,
    pub created_at: DateTime<FixedOffset>,
    pub activated_at: DateTime<FixedOffset>,
    pub retired_at: DateTime<FixedOffset>,
    pub expires_at: DateTime<FixedOffset>,
    pub revoked_at: Option<DateTime<FixedOffset>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RotateKeyResponse {
    pub key: ApplicationKeyVO,
}

fn forbid_search_wildcards(value: &Option<String>, _: &()) -> garde::Result {
    if let Some(value) = value.as_deref()
        && (value.contains('%') || value.contains('_') || value.contains('\\'))
    {
        return Err(garde::Error::new(
            "must not contain expressions that expand `LIKE/ILIKE` search scope",
        ));
    }

    Ok(())
}
