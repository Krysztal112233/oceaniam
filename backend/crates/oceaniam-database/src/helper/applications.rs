use crate::error::Error;
use oceaniam_common::consts;
use oceaniam_vo::pagination::{PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    QuerySelect, sea_query::Expr,
};
use uuid::Uuid;

use crate::{
    config::application::ApplicationConfiguration,
    helper::{
        PagedExecutor, PagedSelect, SafeTransactionConnectionTrait, db_clock_now,
        exactly_one_returned,
    },
    model::{self, prelude::Applications},
};

#[derive(Debug, Default)]
pub struct CreateApplicationOptions {
    pub comment: Option<String>,
    pub configuration: ApplicationConfiguration,
}

#[async_trait::async_trait]
pub trait ApplicationHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.applications.create_application",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_application(
        id: Uuid,
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        Self::create_with_opts(id, tenant_id, CreateApplicationOptions::default(), database).await
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications.create_with_opts",
        skip_all,
        fields(otel.kind = "internal")
    )]
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
            created_at: db_clock_now(database).await?,
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications.is_exist",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications.is_system_application_exist",
        skip_all,
        fields(otel.kind = "internal")
    )]
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
    #[tracing::instrument(
        level = "info",
        name = "db.applications.get_applications",
        skip_all,
        fields(otel.kind = "internal")
    )]
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
    #[tracing::instrument(
        level = "info",
        name = "db.applications.get_all_application_ids",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications.get_application",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications.get_system_application",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications.delete_application",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    /// Replaces an application's stored configuration document.
    ///
    /// The write is a primary-key-filtered conditional update that keeps the system
    /// application excluded and reports [`application_not_found`] when the row is gone instead
    /// of silently succeeding.
    #[tracing::instrument(
        level = "info",
        name = "db.applications.replace_configuration",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn replace_configuration(
        application_id: Uuid,
        configuration: ApplicationConfiguration,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        use model::applications::Column::*;

        let configuration = serde_json::to_value(configuration)?;
        let updated = Applications::update_many()
            .filter(Id.eq(application_id))
            .filter(Id.ne(consts::SYSTEM_APPLICATION_UUID))
            .col_expr(Configuration, Expr::value(configuration))
            .exec_with_returning(database)
            .await?;

        exactly_one_returned(updated, || application_not_found(application_id))
    }

    /// Updates an application's comment.
    ///
    /// The write is a primary-key-filtered conditional update that keeps the system
    /// application excluded and reports [`application_not_found`] when the row is gone instead
    /// of silently succeeding.
    #[tracing::instrument(
        level = "info",
        name = "db.applications.update_comment",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn update_comment(
        application_id: Uuid,
        comment: Option<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::applications::Model, Error> {
        use model::applications::Column::*;

        let updated = Applications::update_many()
            .filter(Id.eq(application_id))
            .filter(Id.ne(consts::SYSTEM_APPLICATION_UUID))
            .col_expr(Comment, Expr::value(comment))
            .exec_with_returning(database)
            .await?;

        exactly_one_returned(updated, || application_not_found(application_id))
    }
}

impl ApplicationHelper for Applications {}

fn is_system_tenant(tenant_id: Uuid) -> bool {
    tenant_id == consts::SYSTEM_TENANT_UUID
}

fn application_not_found(application_id: Uuid) -> Error {
    Error::not_found(format!("application_id={application_id} not found"))
}

impl From<model::applications::Model> for ApplicationConfiguration {
    fn from(value: model::applications::Model) -> Self {
        serde_json::from_value(value.configuration).unwrap()
    }
}
