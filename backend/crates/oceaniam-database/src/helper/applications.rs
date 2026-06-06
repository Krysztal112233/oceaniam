use crate::error::Error;
use axum::http::StatusCode;
use oceaniam_common::consts;
use oceaniam_vo::pagination::{PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QuerySelect,
};
use uuid::Uuid;

use crate::{
    config::application::ApplicationConfiguration,
    helper::{PagedExecutor, PagedSelect, SafeTransactionConnectionTrait},
    model::{self, prelude::Applications},
};

#[derive(Debug, Default)]
pub struct CreateApplicationOptions {
    pub comment: Option<String>,
    pub configuration: ApplicationConfiguration,
}

#[async_trait::async_trait]
pub trait ApplicationHelper {
    async fn create_application(
        id: Uuid,
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        Self::create_with_opts(id, tenant_id, CreateApplicationOptions::default(), database).await
    }

    async fn create_with_opts(
        id: Uuid,
        tenant_id: Uuid,

        opts: CreateApplicationOptions,

        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        let CreateApplicationOptions {
            comment,
            configuration,
        } = opts;

        Ok(model::applications::Model {
            id,
            tenant_id,
            comment,
            configuration: serde_json::to_value(configuration)?,
            created_at: chrono::Utc::now().into(),
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    async fn is_exist(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        crate::system_protected_is_exist!(
            Applications,
            consts::SYSTEM_APPLICATION_UUID,
            id,
            database
        )
    }

    async fn is_system_application_exist(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        crate::system_protected_is_system_exist!(
            Applications,
            consts::SYSTEM_APPLICATION_UUID,
            database
        )
    }

    /// NOTE: This helper intentionally excludes the system tenant and system
    /// application from regular application listing.
    async fn get_applications(
        tenant_id: Uuid,
        page: Option<PageParam>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::applications::Model>, Error> {
        use crate::model::applications::Column::*;

        if is_system_tenant(tenant_id) {
            return Ok(PagedResponse::default());
        }

        let query = Applications::find()
            .filter(TenantId.eq(tenant_id))
            .filter(Id.ne(consts::SYSTEM_APPLICATION_UUID));

        let Some(page) = page else {
            return query
                .all(database)
                .await
                .map(PagedResponse::with_entire)
                .map_err(Into::into);
        };

        query
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    /// NOTE: This helper intentionally excludes identifiers that belong to the
    /// internal system tenant or system application.
    async fn get_all_application_ids(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<Uuid>, Error> {
        use crate::model::applications::Column::*;

        Ok(Applications::find()
            .select_only()
            .column(Id)
            .filter(Id.ne(consts::SYSTEM_APPLICATION_UUID))
            .filter(TenantId.ne(consts::SYSTEM_TENANT_UUID))
            .distinct()
            .into_tuple::<Uuid>()
            .all(database)
            .await?)
    }

    async fn get_application(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        crate::system_protected_get!(
            Applications,
            consts::SYSTEM_APPLICATION_UUID,
            id,
            database,
            application_not_found
        )
    }

    async fn get_system_application(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        crate::system_protected_get_system!(
            Applications,
            consts::SYSTEM_APPLICATION_UUID,
            database,
            application_not_found
        )
    }

    async fn delete_application(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        crate::system_protected_delete!(
            Applications,
            consts::SYSTEM_APPLICATION_UUID,
            id,
            database,
            application_not_found
        )
    }

    async fn replace_configuration(
        application_id: Uuid,
        configuration: ApplicationConfiguration,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        let mut model = Self::get_application(application_id, database)
            .await?
            .into_active_model();
        model.configuration = Set(serde_json::to_value(configuration)?);

        Ok(model.update(database).await?)
    }

    async fn update_comment(
        application_id: Uuid,
        comment: Option<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        let mut model = Self::get_application(application_id, database)
            .await?
            .into_active_model();
        model.comment = Set(comment);

        Ok(model.update(database).await?)
    }
}

impl ApplicationHelper for Applications {}

fn is_system_tenant(tenant_id: Uuid) -> bool {
    tenant_id == consts::SYSTEM_TENANT_UUID
}

fn application_not_found(application_id: Uuid) -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        format!("application_id={application_id} not found"),
    )
}

impl From<model::applications::Model> for ApplicationConfiguration {
    fn from(value: model::applications::Model) -> Self {
        serde_json::from_value(value.configuration).unwrap()
    }
}
