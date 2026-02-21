use oceaniam_common::types::sqid::Sqid;
use oceaniam_database::model;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, Serialize, ts_rs::TS)]
pub struct GetUsersRequest {
    pub application_id: Sqid,
}

#[derive(Debug, Deserialize, ToSchema, Serialize, ts_rs::TS)]
pub struct UserVO {
    pub id: Sqid,
    pub application_id: Sqid,
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl From<model::users::Model> for UserVO {
    fn from(
        model::users::Model {
            id,
            application_id,
            email,
            phone,
        }: model::users::Model,
    ) -> Self {
        Self {
            id: id.into(),
            application_id: application_id.into(),
            email,
            phone,
        }
    }
}
