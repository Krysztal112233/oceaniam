use core::str;

use garde::Validate;
use oceaniam_common::{PageParam, types::sqid::Sqid};
use oceaniam_database::model::{self};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct GetApplicationParam {
    #[garde(skip)]
    pub tenant_id: Sqid,

    #[serde(flatten)]
    #[garde(dive)]
    pub page: Option<PageParam>,
}

#[derive(Debug, Deserialize, ToSchema, ts_rs::TS)]
pub struct CreateApplicationRequest {
    pub tenant_id: Sqid,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchAuthenticationConfigurationVO {
    pub issuer: Option<String>,
    pub audience: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default, ts_rs::TS, ToSchema)]
pub struct PatchApplicationConfigurationRequest {
    pub authentication: Option<PatchAuthenticationConfigurationVO>,
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

impl From<oceaniam_database::helper::applications::ApplicationConfiguration>
    for ApplicationConfigurationVO
{
    fn from(
        oceaniam_database::helper::applications::ApplicationConfiguration { authentication }:
            oceaniam_database::helper::applications::ApplicationConfiguration,
    ) -> Self {
        Self {
            authentication: authentication.into(),
        }
    }
}

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

    /// User nickname (optional, if not provided, two random words will be generated)
    #[serde(default = "oceaniam_common::helpers::gen_random_name")]
    #[garde(skip)]
    pub nickname: String,

    #[garde(skip)]
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct ApplicationUserVO {
    pub id: Sqid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: String,
}

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
}

impl SecretVO {
    pub fn with_masked(model: model::application_secrets::Model) -> Self {
        Self {
            secret: format!("{}...", model.secret.clone().split_at(8).0),
            ..Self::with_unmasked(model)
        }
    }

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
        }
    }
}
