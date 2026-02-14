use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema, ts_rs::TS)]
pub struct CreateApplicationRequest {
    pub tenant_id: Uuid,
}
