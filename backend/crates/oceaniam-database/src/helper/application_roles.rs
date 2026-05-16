use axum::http::StatusCode;
use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::{
    error::Error, helper::SafeTransactionConnectionTrait, model::prelude::ApplicationRoles,
};

#[async_trait::async_trait]
pub trait ApplicationRolesHelper {
    async fn resolve_role_name(
        role_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<String, Error> {
        ApplicationRoles::find_by_id(role_id)
            .one(database)
            .await?
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("application_role {role_id} not found"),
                )
            })
            .map(|r| r.name)
    }
}

impl ApplicationRolesHelper for ApplicationRoles {}
