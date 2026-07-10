use std::collections::HashSet;

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AdministratorVO {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
pub struct CreateAdministratorRequest {
    #[garde(length(min = 3))]
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Validate, Deserialize, ToSchema)]
pub struct PatchAdministratorRequest {
    #[garde(length(min = 3))]
    pub name: Option<String>,
    #[garde(length(min = 12))]
    pub password: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateAdministratorResponse {
    pub administrator: AdministratorVO,
    pub initial_password: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AdministratorProfileVO {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub permissions: HashSet<String>,
}
