use oceaniam_common::types::sqid::Sqid;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, ts_rs::TS)]
pub struct CreateApplicationRequest {
    pub tenant_id: Sqid,
}
