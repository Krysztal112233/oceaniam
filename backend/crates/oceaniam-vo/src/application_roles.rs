use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct ApplicationRoleVO {
    pub id: Uuid,
    pub application_id: Uuid,
    pub name: String,
    pub is_system: bool,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct CreateApplicationRoleRequest {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct PatchApplicationRoleRequest {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct RolePermissionsVO {
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct SetRolePermissionsRequest {
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct SubjectRolesVO {
    pub subject_id: Uuid,
    pub role_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}
