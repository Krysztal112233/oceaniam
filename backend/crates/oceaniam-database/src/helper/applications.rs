use axum::http::StatusCode;
use oceaniam_common::{PageParam, PagedResponse, consts, error::Error};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
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
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    /// NOTE: This helper intentionally treats the system application as not
    /// visible from the regular application access path.
    async fn is_exist(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        if is_system_application(id) {
            return Ok(false);
        }

        Ok(Applications::find_by_id(id).one(database).await?.is_some())
    }

    /// NOTE: This helper is reserved for internal bootstrap and system-only
    /// code paths that need the real existence of the reserved system
    /// application.
    async fn is_system_application_exist(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        Ok(Applications::find_by_id(consts::SYSTEM_APPLICATION_UUID)
            .one(database)
            .await?
            .is_some())
    }

    /// NOTE: This helper intentionally excludes the system tenant and system
    /// application from regular application listing.
    async fn get_applications(
        tenant_id: Uuid,
        page: impl Into<PageParam> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::applications::Model>, Error> {
        use crate::model::applications::Column::*;

        let page = page.into();

        if is_system_tenant(tenant_id) {
            return Ok(PagedResponse::default());
        }

        Applications::find()
            .filter(TenantId.eq(tenant_id))
            .filter(Id.ne(consts::SYSTEM_APPLICATION_UUID))
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    /// NOTE: This helper intentionally excludes the system tenant and system
    /// application from regular application listing.
    async fn get_all_applications(
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::applications::Model>, Error> {
        use crate::model::applications::Column::*;

        if is_system_tenant(tenant_id) {
            return Ok(Vec::new());
        }

        Ok(Applications::find()
            .filter(TenantId.eq(tenant_id))
            .filter(Id.ne(consts::SYSTEM_APPLICATION_UUID))
            .all(database)
            .await?)
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

    /// NOTE: This helper intentionally blocks direct reads of the system
    /// application from the regular application access path.
    async fn get_application(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        if is_system_application(application_id) {
            return Err(application_not_found(application_id));
        }

        Applications::find_by_id(application_id)
            .one(database)
            .await
            .map(|it| it.ok_or(application_not_found(application_id)))?
    }

    /// NOTE: This is the explicit escape hatch for internal code that still
    /// needs to access the reserved system application record.
    async fn get_system_application(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        Applications::find_by_id(consts::SYSTEM_APPLICATION_UUID)
            .one(database)
            .await
            .map(|it| it.ok_or(application_not_found(consts::SYSTEM_APPLICATION_UUID)))?
    }

    /// NOTE: This helper intentionally blocks deletion of the reserved system
    /// application through the regular application lifecycle path.
    async fn delete_application(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        if is_system_application(application_id) {
            return Err(application_not_found(application_id));
        }

        Applications::delete_by_id(application_id)
            .exec(database)
            .await?;

        Ok(())
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

fn is_system_application(application_id: Uuid) -> bool {
    application_id == consts::SYSTEM_APPLICATION_UUID
}

fn is_system_tenant(tenant_id: Uuid) -> bool {
    tenant_id == consts::SYSTEM_TENANT_UUID
}

fn application_not_found(application_id: Uuid) -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        format!("application_id={application_id} not found"),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplicationConfiguration {
    pub authentication: AuthenticationConfiguration,
    pub enable_registration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfiguration {
    pub issuer: String,
    pub audience: Vec<String>,
}

impl Default for AuthenticationConfiguration {
    fn default() -> Self {
        Self {
            issuer: consts::DEFAULT_JWT_ISSUER.to_owned(),
            audience: vec![consts::DEFAULT_JWT_AUDIENCE.to_owned()],
        }
    }
}

impl From<model::applications::Model> for ApplicationConfiguration {
    fn from(value: model::applications::Model) -> Self {
        serde_json::from_value(value.configuration).unwrap()
    }
}
