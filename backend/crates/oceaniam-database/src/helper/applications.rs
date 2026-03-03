use axum::http::StatusCode;
use oceaniam_common::{PageParam, PagedResponse, consts, error::Error};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
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

    async fn is_exist(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        Ok(Applications::find_by_id(id).one(database).await?.is_some())
    }

    async fn get_applications(
        tenant_id: Uuid,
        page: impl Into<PageParam> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::applications::Model>, Error> {
        use crate::model::applications::Column::*;

        let page = page.into();

        Applications::find()
            .filter(TenantId.eq(tenant_id))
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    async fn get_all_applications(
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::applications::Model>, Error> {
        use crate::model::applications::Column::*;

        Ok(Applications::find()
            .filter(TenantId.eq(tenant_id))
            .all(database)
            .await?)
    }

    async fn get_application(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        Applications::find_by_id(application_id)
            .one(database)
            .await
            .map(|it| {
                it.ok_or(Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("application_id={application_id} not found"),
                ))
            })?
    }

    async fn delete_application(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        Applications::delete_by_id(application_id)
            .exec(database)
            .await?;

        Ok(())
    }
}

impl ApplicationHelper for Applications {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplicationConfiguration {
    pub authentication: AuthenticationConfiguration,
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
            audience: Vec::new(),
        }
    }
}

impl From<model::applications::Model> for ApplicationConfiguration {
    fn from(value: model::applications::Model) -> Self {
        serde_json::from_value(value.configuration).unwrap()
    }
}
