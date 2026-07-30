use axum::http::StatusCode;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::ApplicationRoles},
};

#[async_trait::async_trait]
pub trait ApplicationRolesHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.application_roles.resolve_role_name",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.application_roles.get_role_by_id",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_role_by_id(
        role_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_roles::Model, Error> {
        ApplicationRoles::find_by_id(role_id)
            .one(database)
            .await?
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("application_role {role_id} not found"),
                )
            })
    }

    #[tracing::instrument(
        level = "info",
        name = "db.application_roles.get_roles_by_application",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_roles_by_application(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::application_roles::Model>, Error> {
        Ok(ApplicationRoles::find()
            .filter(model::application_roles::Column::ApplicationId.eq(application_id))
            .all(database)
            .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.application_roles.create_role",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_role(
        id: Uuid,
        application_id: Uuid,
        name: String,
        is_system: bool,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_roles::Model, Error> {
        Ok(model::application_roles::ActiveModel {
            id: Set(id),
            application_id: Set(application_id),
            name: Set(name),
            is_system: Set(is_system),
        }
        .insert(database)
        .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.application_roles.update_role_name",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn update_role_name(
        role_id: Uuid,
        name: String,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_roles::Model, Error> {
        let mut role = Self::get_role_by_id(role_id, database)
            .await?
            .into_active_model();
        role.name = Set(name);
        Ok(role.update(database).await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.application_roles.delete_role",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn delete_role(
        role_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        ApplicationRoles::delete_by_id(role_id)
            .exec(database)
            .await?;
        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "db.application_roles.get_roles_by_ids",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_roles_by_ids(
        role_ids: Vec<Uuid>,
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::application_roles::Model>, Error> {
        Ok(ApplicationRoles::find()
            .filter(model::application_roles::Column::Id.is_in(role_ids))
            .filter(model::application_roles::Column::ApplicationId.eq(application_id))
            .all(database)
            .await?)
    }
}

impl ApplicationRolesHelper for ApplicationRoles {}
