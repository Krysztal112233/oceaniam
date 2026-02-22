use chrono::Utc;
use oceaniam_common::error::Error;
use oceaniam_common::{PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
};
use uuid::Uuid;

use crate::helper::{PagedExecutor, PagedSelect};
use crate::model::prelude::ApplicationSecrets;
use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self},
};

#[async_trait::async_trait]
pub trait ApplicationSecretsHelper {
    async fn create_secret(
        application_id: Uuid,
        id: Uuid,
        secret: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_secrets::Model, Error> {
        Ok(model::application_secrets::Model {
            id,
            application_id,
            secret: secret.into(),
            created_at: Utc::now().into(),
            revoked_at: None,
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    async fn get_secrets(
        application_id: Uuid,
        paged: Option<PageParam>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::application_secrets::Model>, Error> {
        use model::application_secrets::Column::*;

        match paged {
            Some(paged) => {
                ApplicationSecrets::find()
                    .filter(ApplicationId.eq(application_id))
                    .paged(paged)
                    .paginate(database, paged.per_page)
                    .fetch_paged(paged)
                    .await
            }
            None => Ok(ApplicationSecrets::find()
                .filter(ApplicationId.eq(application_id))
                .all(database)
                .await
                .map(PagedResponse::with_entire)?),
        }
    }
}

impl ApplicationSecretsHelper for ApplicationSecrets {}
