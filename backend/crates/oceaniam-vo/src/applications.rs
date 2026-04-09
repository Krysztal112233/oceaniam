use core::str;

use garde::Validate;
use oceaniam_common::types::sqid::Sqid;
#[cfg(feature = "database")]
use oceaniam_database::model::{self};
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;

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
pub struct AuthenticationConfigurationVO {
    pub issuer: String,
    pub audience: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct ApplicationConfigurationVO {
    pub authentication: AuthenticationConfigurationVO,
    pub enable_registration: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchAuthenticationConfigurationVO {
    pub issuer: Option<String>,
    pub audience: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchApplicationConfigurationRequest {
    pub authentication: Option<PatchAuthenticationConfigurationVO>,
    pub enable_registration: Option<bool>,
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
impl From<oceaniam_database::helper::applications::AuthenticationConfiguration>
    for AuthenticationConfigurationVO
{
    fn from(
        oceaniam_database::helper::applications::AuthenticationConfiguration {
            issuer,
            audience,
        }: oceaniam_database::helper::applications::AuthenticationConfiguration,
    ) -> Self {
        Self { issuer, audience }
    }
}

#[cfg(feature = "database")]
impl From<oceaniam_database::helper::applications::ApplicationConfiguration>
    for ApplicationConfigurationVO
{
    fn from(
        oceaniam_database::helper::applications::ApplicationConfiguration {
            authentication,
            enable_registration: allow_registration,
        }: oceaniam_database::helper::applications::ApplicationConfiguration,
    ) -> Self {
        Self {
            authentication: authentication.into(),
            enable_registration: allow_registration,
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
                oceaniam_database::helper::applications::ApplicationConfiguration,
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Validate, Deserialize, ts_rs::TS, ToSchema)]
#[serde(default)]
pub struct SearchApplicationUsersQuery {
    #[garde(custom(forbid_search_wildcards))]
    pub by_nickname: Option<String>,
    #[garde(custom(forbid_search_wildcards))]
    pub by_email: Option<String>,
    #[garde(custom(forbid_search_wildcards))]
    pub by_phone: Option<String>,
    #[garde(skip)]
    pub by_id: Option<Sqid>,
}

impl SearchApplicationUsersQuery {
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
