use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, Serialize)]
pub struct CreateTenantRequest {
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Serialize)]
pub struct PatchTenantRequest {
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Serialize)]
pub struct TenantVO {
    pub id: String,
    pub comment: Option<String>,
}
