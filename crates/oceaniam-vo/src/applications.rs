use oceaniam_common::{PageParam, types::sqid::Sqid};
use oceaniam_database::model::{self};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetApplicationParam {
    pub tenant_id: Sqid,

    pub page: u64,
    pub per_page: u64,
}

impl From<GetApplicationParam> for PageParam {
    fn from(GetApplicationParam { page, per_page, .. }: GetApplicationParam) -> Self {
        Self { page, per_page }
    }
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
        }: model::applications::Model,
    ) -> Self {
        Self {
            id: id.into(),
            comment,
            tenant_id: tenant_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct SecretVO {
    pub id: Sqid,
    pub created_at: String,
    pub revoked_at: Option<String>,
}
