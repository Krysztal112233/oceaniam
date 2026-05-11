use core::str;

use crate::sqid::Sqid;
use chrono::{DateTime, FixedOffset};
use garde::Validate;
use oceaniam_api::PageParam;
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;

#[cfg(feature = "database")]
use oceaniam_database::model::{self};

#[derive(Debug, Deserialize, ToSchema, ts_rs::TS)]
pub struct CreateApplicationRequest {
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, ts_rs::TS)]
pub struct CreateApplicationResponse {
    pub tenant_id: Sqid,
    pub application_id: Sqid,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct TokenConfigurationVO {
    pub issuer: String,
    pub audience: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ts_rs::TS, ToSchema)]
pub struct Argon2Configuration {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct PasswordConfigurationVO {
    pub argon2: Argon2Configuration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct AuthConfigurationVO {
    pub token: TokenConfigurationVO,
    pub password: PasswordConfigurationVO,
    pub totp: TotpConfigurationVO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct TotpConfigurationVO {
    pub encryption_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct RegistrationConfigurationVO {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct ApplicationConfigurationVO {
    pub auth: AuthConfigurationVO,
    pub registration: RegistrationConfigurationVO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchTokenConfigurationVO {
    pub issuer: Option<String>,
    pub audience: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchAuthConfigurationVO {
    pub token: Option<PatchTokenConfigurationVO>,
    pub totp: Option<PatchTotpConfigurationVO>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchTotpConfigurationVO {
    pub encryption_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchRegistrationConfigurationVO {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchApplicationConfigurationRequest {
    pub auth: Option<PatchAuthConfigurationVO>,
    pub registration: Option<PatchRegistrationConfigurationVO>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum PatchValue<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<'de, T> Deserialize<'de> for PatchValue<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, ToSchema)]
pub struct PatchApplicationRequest {
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    pub comment: PatchValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct GetApplicationConfigurationResponse {
    pub configuration: ApplicationConfigurationVO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs :: TS, ToSchema)]
pub struct ApplicationDetailVO {
    pub id: Sqid,
    pub comment: Option<String>,
    pub tenant_id: Sqid,
    pub configuration: ApplicationConfigurationVO,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs :: TS, ToSchema)]
pub struct ApplicationVO {
    pub id: Sqid,
    pub comment: Option<String>,
    pub tenant_id: Sqid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct ApplicationChallengeVO {
    pub id: uuid::Uuid,
    pub application_id: Sqid,
    pub subject_id: uuid::Uuid,
    pub factor_type: String,
    pub purpose: String,
    pub status: String,
    pub attempt_count: i32,
    pub remaining_attempts: i32,
    pub expires_at: DateTime<FixedOffset>,
    pub consumed_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
}

#[cfg(feature = "database")]
impl From<model::applications::Model> for ApplicationVO {
    fn from(
        model::applications::Model {
            id,
            comment,
            tenant_id,
            ..
        }: model::applications::Model,
    ) -> Self {
        Self {
            id: id.into(),
            comment,
            tenant_id: tenant_id.into(),
        }
    }
}

#[cfg(feature = "database")]
impl From<model::challenges::Model> for ApplicationChallengeVO {
    fn from(value: model::challenges::Model) -> Self {
        Self {
            id: value.id,
            application_id: value.application_id.into(),
            subject_id: value.subject_id,
            factor_type: value.factor_type.to_string(),
            purpose: value.purpose.to_string(),
            status: value.status.to_string(),
            attempt_count: value.attempt_count,
            remaining_attempts: (value.max_attempts - value.attempt_count).max(0),
            expires_at: value.expires_at,
            consumed_at: value.consumed_at,
            created_at: value.created_at,
        }
    }
}

#[cfg(feature = "database")]
impl From<oceaniam_database::config::application::TokenConfiguration> for TokenConfigurationVO {
    fn from(
        oceaniam_database::config::application::TokenConfiguration {
            issuer,
            audience,
        }: oceaniam_database::config::application::TokenConfiguration,
    ) -> Self {
        Self { issuer, audience }
    }
}

#[cfg(feature = "database")]
impl From<oceaniam_database::config::application::Argon2Configuration> for Argon2Configuration {
    fn from(
        oceaniam_database::config::application::Argon2Configuration {
            m_cost,
            t_cost,
            p_cost,
        }: oceaniam_database::config::application::Argon2Configuration,
    ) -> Self {
        Self {
            m_cost,
            t_cost,
            p_cost,
        }
    }
}

#[cfg(feature = "database")]
impl From<oceaniam_database::config::application::PasswordConfiguration>
    for PasswordConfigurationVO
{
    fn from(
        oceaniam_database::config::application::PasswordConfiguration { argon2 }:
            oceaniam_database::config::application::PasswordConfiguration,
    ) -> Self {
        Self {
            argon2: argon2.into(),
        }
    }
}

#[cfg(feature = "database")]
impl From<oceaniam_database::config::application::AuthConfiguration> for AuthConfigurationVO {
    fn from(
        oceaniam_database::config::application::AuthConfiguration {
            token,
            password,
            totp,
        }: oceaniam_database::config::application::AuthConfiguration,
    ) -> Self {
        Self {
            token: token.into(),
            password: password.into(),
            totp: totp.into(),
        }
    }
}

#[cfg(feature = "database")]
impl From<oceaniam_database::config::application::TotpConfiguration> for TotpConfigurationVO {
    fn from(
        oceaniam_database::config::application::TotpConfiguration { encryption_key }:
            oceaniam_database::config::application::TotpConfiguration,
    ) -> Self {
        Self { encryption_key }
    }
}

#[cfg(feature = "database")]
impl From<oceaniam_database::config::application::RegistrationConfiguration>
    for RegistrationConfigurationVO
{
    fn from(
        oceaniam_database::config::application::RegistrationConfiguration { enabled }:
            oceaniam_database::config::application::RegistrationConfiguration,
    ) -> Self {
        Self { enabled }
    }
}

#[cfg(feature = "database")]
impl From<oceaniam_database::config::application::ApplicationConfiguration>
    for ApplicationConfigurationVO
{
    fn from(
        oceaniam_database::config::application::ApplicationConfiguration {
            auth,
            registration,
        }: oceaniam_database::config::application::ApplicationConfiguration,
    ) -> Self {
        Self {
            auth: auth.into(),
            registration: registration.into(),
        }
    }
}

#[cfg(feature = "database")]
impl From<model::applications::Model> for ApplicationDetailVO {
    fn from(
        model::applications::Model {
            id,
            comment,
            tenant_id,
            configuration,
        }: model::applications::Model,
    ) -> Self {
        Self {
            id: id.into(),
            comment,
            tenant_id: tenant_id.into(),
            configuration: serde_json::from_value::<
                oceaniam_database::config::application::ApplicationConfiguration,
            >(configuration)
            .unwrap()
            .into(),
        }
    }
}

/// VO for creating a new application user
///
/// Either `phone` or `email` must be provided (mutually exclusive)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ts_rs::TS, ToSchema)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ts_rs::TS, ToSchema)]
pub struct PatchApplicationUserCredentialsRequest {
    #[garde(length(min = 12))]
    pub password: Option<String>,
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

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationUsersSortOrder {
    Asc,

    #[default]
    Desc,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ts_rs::TS, ToSchema,
)]
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
        let page = PageParam::default();

        Self {
            page: page.page,
            per_page: page.per_page,
            sort_order: ApplicationUsersSortOrder::default(),
        }
    }
}

impl ApplicationUsersListQuery {
    pub fn page_param(&self) -> PageParam {
        PageParam {
            page: self.page,
            per_page: self.per_page,
        }
    }

    pub fn is_desc(&self) -> bool {
        matches!(self.sort_order, ApplicationUsersSortOrder::Desc)
    }
}

impl From<ApplicationUsersListQuery> for PageParam {
    fn from(value: ApplicationUsersListQuery) -> Self {
        value.page_param()
    }
}

impl From<PageParam> for ApplicationUsersListQuery {
    fn from(value: PageParam) -> Self {
        Self {
            page: value.page,
            per_page: value.per_page,
            sort_order: ApplicationUsersSortOrder::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ts_rs::TS, ToSchema)]
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
    pub by_id: Option<Sqid>,
}

impl Default for SearchApplicationUsersQuery {
    fn default() -> Self {
        let page = PageParam::default();

        Self {
            page: page.page,
            per_page: page.per_page,
            sort_order: ApplicationUsersSortOrder::default(),
            by_nickname: None,
            by_email: None,
            by_phone: None,
            by_id: None,
        }
    }
}

impl SearchApplicationUsersQuery {
    pub fn page_param(&self) -> PageParam {
        PageParam {
            page: self.page,
            per_page: self.per_page,
        }
    }

    pub fn is_desc(&self) -> bool {
        matches!(self.sort_order, ApplicationUsersSortOrder::Desc)
    }

    pub fn has_search_term(&self) -> bool {
        self.by_nickname
            .as_deref()
            .map(str::trim)
            .filter(|it| !it.is_empty())
            .is_some()
            || self
                .by_email
                .as_deref()
                .map(str::trim)
                .filter(|it| !it.is_empty())
                .is_some()
            || self
                .by_phone
                .as_deref()
                .map(str::trim)
                .filter(|it| !it.is_empty())
                .is_some()
            || self.by_id.is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct ApplicationUserVO {
    pub id: Sqid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: String,
}

#[cfg(feature = "database")]
impl From<oceaniam_database::model::users::Model> for ApplicationUserVO {
    fn from(
        oceaniam_database::model::users::Model {
            id,
            email,
            phone,
            nickname,
            ..
        }: oceaniam_database::model::users::Model,
    ) -> Self {
        Self {
            id: id.into(),
            email,
            phone,
            nickname,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct SecretVO {
    pub id: Sqid,
    pub secret: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
    pub application_ids: Vec<Sqid>,
}

impl SecretVO {
    #[cfg(feature = "database")]
    pub fn with_masked(model: model::application_secrets::Model) -> Self {
        Self {
            secret: format!("{}...", model.secret.clone().split_at(8).0),
            ..Self::with_unmasked(model)
        }
    }

    #[cfg(feature = "database")]
    pub fn with_unmasked(
        model::application_secrets::Model {
            id,
            secret,
            created_at,
            revoked_at,
            ..
        }: model::application_secrets::Model,
    ) -> Self {
        Self {
            id: id.into(),
            secret,
            created_at: created_at.to_rfc2822(),
            revoked_at: revoked_at.map(|it| it.to_rfc2822()),
            application_ids: Vec::new(),
        }
    }

    pub fn with_application_ids(
        mut self,
        application_ids: impl IntoIterator<Item = uuid::Uuid>,
    ) -> Self {
        self.application_ids = application_ids.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct ApplicationKeyVO {
    pub key_id: Sqid,
    pub algorithm: String,
    pub status: String,
    pub created_at: DateTime<FixedOffset>,
    pub activated_at: Option<DateTime<FixedOffset>>,
    pub retired_at: Option<DateTime<FixedOffset>>,
    pub expires_at: Option<DateTime<FixedOffset>>,
    pub revoked_at: Option<DateTime<FixedOffset>>,
}

#[cfg(feature = "database")]
impl From<oceaniam_database::model::key_boxes::Model> for ApplicationKeyVO {
    fn from(
        oceaniam_database::model::key_boxes::Model {
            id,
            key_alg,
            status,
            created_at,
            activated_at,
            retired_at,
            expires_at,
            revoked_at,
            ..
        }: oceaniam_database::model::key_boxes::Model,
    ) -> Self {
        Self {
            key_id: id.into(),
            algorithm: key_alg.to_string(),
            status: status.to_string(),
            created_at,
            activated_at,
            retired_at,
            expires_at,
            revoked_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct RotateKeyResponse {
    pub key: ApplicationKeyVO,
}
