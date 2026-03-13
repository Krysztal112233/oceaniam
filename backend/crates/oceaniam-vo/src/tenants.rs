use oceaniam_common::{PageParam, types::sqid::Sqid};
use oceaniam_database::model;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, Serialize, ts_rs::TS)]
pub struct CreateTenantRequest {
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Serialize, ts_rs::TS)]
pub struct TenantVO {
    pub id: Sqid,
    pub comment: Option<String>,
}

impl From<model::tenants::Model> for TenantVO {
    fn from(model::tenants::Model { id, comment }: model::tenants::Model) -> Self {
        Self {
            id: id.into(),
            comment,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, Serialize, ts_rs::TS)]
pub struct GetTenantsRequest {
    pub page: u64,
    pub per_page: u64,
}

impl From<GetTenantsRequest> for PageParam {
    fn from(GetTenantsRequest { page, per_page }: GetTenantsRequest) -> Self {
        Self { page, per_page }
    }
}

impl Default for GetTenantsRequest {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: 30,
        }
    }
}
