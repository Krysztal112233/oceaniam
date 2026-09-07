use axum::http::StatusCode;
use oceaniam_vo::pagination::{PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::{PagedExecutor, PagedSelect, SafeTransactionConnectionTrait},
    model::{
        self,
        prelude::{OidcClientRedirectUris, OidcClients},
        sea_orm_active_enums::{OidcApplicationType, OidcClientType},
    },
};

#[async_trait::async_trait]
pub trait OidcClientsHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.create",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_client(
        id: Uuid,
        application_id: Uuid,
        client_id: String,
        name: String,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_clients::Model, Error> {
        Ok(model::oidc_clients::ActiveModel {
            id: Set(id),
            application_id: Set(application_id),
            client_id: Set(client_id),
            name: Set(name),
            client_type: Set(OidcClientType::Public),
            application_type: Set(OidcApplicationType::Web),
            created_at: Set(chrono::Utc::now().into()),
        }
        .insert(database)
        .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.create_redirect_uris",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_redirect_uris(
        oidc_client_id: Uuid,
        redirect_uris: Vec<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::oidc_client_redirect_uris::Model>, Error> {
        let now = chrono::Utc::now().into();
        let models = redirect_uris
            .into_iter()
            .map(
                |redirect_uri| model::oidc_client_redirect_uris::ActiveModel {
                    id: Set(Uuid::now_v7()),
                    oidc_client_id: Set(oidc_client_id),
                    redirect_uri: Set(redirect_uri),
                    created_at: Set(now),
                },
            )
            .collect::<Vec<_>>();

        Ok(OidcClientRedirectUris::insert_many(models)
            .exec_with_returning(database)
            .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.list",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_clients(
        application_id: Uuid,
        page: PageParam,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::oidc_clients::Model>, Error> {
        use model::oidc_clients::Column::*;

        OidcClients::find()
            .filter(ApplicationId.eq(application_id))
            .order_by_desc(CreatedAt)
            .order_by_desc(Id)
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.get",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_client(
        application_id: Uuid,
        client_id: &str,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_clients::Model, Error> {
        use model::oidc_clients::Column::*;

        OidcClients::find()
            .filter(ApplicationId.eq(application_id))
            .filter(ClientId.eq(client_id))
            .one(database)
            .await?
            .ok_or_else(oidc_client_not_found)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.get_for_update",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_client_for_update(
        application_id: Uuid,
        client_id: &str,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_clients::Model, Error> {
        use model::oidc_clients::Column::*;

        OidcClients::find()
            .filter(ApplicationId.eq(application_id))
            .filter(ClientId.eq(client_id))
            .lock_exclusive()
            .one(database)
            .await?
            .ok_or_else(oidc_client_not_found)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.update_name",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn update_client_name(
        client: model::oidc_clients::Model,
        name: String,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::oidc_clients::Model, Error> {
        let mut client = client.into_active_model();
        client.name = Set(name);
        Ok(client.update(database).await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.replace_redirect_uris",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn replace_redirect_uris(
        oidc_client_id: Uuid,
        redirect_uris: Vec<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::oidc_client_redirect_uris::Model>, Error> {
        use model::oidc_client_redirect_uris::Column::OidcClientId;

        OidcClientRedirectUris::delete_many()
            .filter(OidcClientId.eq(oidc_client_id))
            .exec(database)
            .await?;
        Self::create_redirect_uris(oidc_client_id, redirect_uris, database).await
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.delete",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn delete_locked_client(
        oidc_client_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let result = OidcClients::delete_by_id(oidc_client_id)
            .exec(database)
            .await?;
        if result.rows_affected != 1 {
            return Err(oidc_client_not_found());
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "db.oidc_clients.get_redirect_uris",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_redirect_uris(
        oidc_client_ids: Vec<Uuid>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::oidc_client_redirect_uris::Model>, Error> {
        use model::oidc_client_redirect_uris::Column::*;

        if oidc_client_ids.is_empty() {
            return Ok(Vec::new());
        }

        Ok(OidcClientRedirectUris::find()
            .filter(OidcClientId.is_in(oidc_client_ids))
            .order_by_asc(OidcClientId)
            .order_by_asc(RedirectUri)
            .all(database)
            .await?)
    }
}

fn oidc_client_not_found() -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        "OIDC client not found in this application",
    )
}

impl OidcClientsHelper for OidcClients {}
