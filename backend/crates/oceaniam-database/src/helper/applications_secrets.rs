use axum::http::StatusCode;
use chrono::Utc;
use oceaniam_common::error::Error;
use oceaniam_common::{PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QuerySelect,
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

    async fn get_all(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::application_secrets::Model>, Error> {
        use model::application_secrets::Column::*;

        Ok(ApplicationSecrets::find()
            .filter(ApplicationId.eq(application_id))
            .all(database)
            .await?)
    }

    async fn find_secret_belong(
        secret: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_secrets::Model, Error> {
        use model::application_secrets::Column::*;

        let secret = secret.into();

        ApplicationSecrets::find()
            .filter(Secret.eq(&secret))
            .one(database)
            .await
            .map(|it| match it {
                Some(it) => Ok(it),
                None => Err(Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("cannot found application_secret={secret}"),
                )),
            })?
    }

    async fn delete_secret(
        application_id: Uuid,
        secret_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        use model::application_secrets::Column::*;

        ApplicationSecrets::find()
            .filter(
                Condition::all()
                    .add(ApplicationId.eq(application_id))
                    .add(Id.eq(secret_id)),
            )
            .all(database)
            .await?;

        Ok(())
    }

    async fn get_all_secret_ids(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<Uuid>, Error> {
        use model::application_secrets::Column::*;

        Ok(ApplicationSecrets::find()
            .select_only()
            .column(Id)
            .distinct()
            .into_tuple::<Uuid>()
            .all(database)
            .await?)
    }

    async fn get_all_secrets(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<String>, Error> {
        use model::application_secrets::Column::*;

        Ok(ApplicationSecrets::find()
            .select_only()
            .column(Secret)
            .distinct()
            .into_tuple::<String>()
            .all(database)
            .await?)
    }
}

impl ApplicationSecretsHelper for ApplicationSecrets {}
