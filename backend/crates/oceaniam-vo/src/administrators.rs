use crate::sqid::Sqid;
use garde::Validate;
#[cfg(feature = "database")]
use oceaniam_database::model;
use oceaniam_permission::Permission;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct AdministratorVO {
    pub id: Sqid,
    pub name: String,
}

#[cfg(feature = "database")]
impl From<model::administrators::Model> for AdministratorVO {
    fn from(model::administrators::Model { id, name, .. }: model::administrators::Model) -> Self {
        Self {
            id: id.into(),
            name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ts_rs::TS, ToSchema)]
pub struct CreateAdministratorRequest {
    #[garde(length(min = 3))]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ts_rs::TS, ToSchema)]
pub struct PatchAdministratorRequest {
    #[garde(length(min = 3))]
    pub name: Option<String>,
    #[garde(length(min = 12))]
    pub password: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, ToSchema)]
pub struct CreateAdministratorResponse {
    pub administrator: AdministratorVO,
    pub initial_password: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AdministratorProfileVO {
    pub id: Sqid,
    pub name: String,
    pub role: Option<String>,
    pub permissions: HashSet<Permission>,
}
