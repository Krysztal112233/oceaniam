use axum::http::StatusCode;
use chrono::Utc;
use oceaniam_common::error::Error;
use oceaniam_common::{PageInfo, PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter,
    QuerySelect,
};
use uuid::Uuid;

use crate::model::prelude::{ApplicationSecretBindings, ApplicationSecrets};
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
        let transaction = database.begin().await?;

        let model = model::application_secrets::Model {
            id,
            secret: secret.into(),
            created_at: Utc::now().into(),
            revoked_at: None,
        }
        .into_active_model()
        .insert(&transaction)
        .await?;

        model::application_secret_bindings::ActiveModel {
            secret_id: sea_orm::ActiveValue::Set(model.id),
            application_id: sea_orm::ActiveValue::Set(application_id),
        }
        .insert(&transaction)
        .await?;

        transaction.commit().await?;

        Ok(model)
    }

    async fn get_secrets(
        application_id: Uuid,
        paged: Option<PageParam>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::application_secrets::Model>, Error> {
        let all_items = Self::get_all(application_id, database).await?;

        match paged {
            Some(paged) => {
                let start = paged.as_offset() as usize;
                let end = start
                    .saturating_add(paged.per_page as usize)
                    .min(all_items.len());
                let items = if start >= all_items.len() {
                    Vec::new()
                } else {
                    all_items[start..end].to_vec()
                };

                Ok(PagedResponse {
                    page_info: PageInfo {
                        has_next: end < all_items.len(),
                        total: all_items.len(),
                    },
                    items,
                })
            }
            None => Ok(PagedResponse::with_entire(all_items)),
        }
    }

    async fn get_all(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::application_secrets::Model>, Error> {
        use model::application_secret_bindings::Column::*;

        Ok(ApplicationSecretBindings::find()
            .filter(ApplicationId.eq(application_id))
            .find_also_related(ApplicationSecrets)
            .all(database)
            .await?
            .into_iter()
            .filter_map(|(_, secret)| secret)
            .collect())
    }

    async fn find_secret_belong(
        secret: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<Uuid>, Error> {
        use model::application_secrets::Column::*;

        let secret = secret.into();
        let secret_id = ApplicationSecrets::find()
            .filter(Secret.eq(&secret))
            .one(database)
            .await
            .map(|it| match it {
                Some(it) => Ok(it.id),
                None => Err(Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("cannot found application_secret={secret}"),
                )),
            })??;

        use model::application_secret_bindings::Column::*;

        Ok(ApplicationSecretBindings::find()
            .filter(SecretId.eq(secret_id))
            .select_only()
            .column(ApplicationId)
            .into_tuple::<Uuid>()
            .all(database)
            .await?)
    }

    async fn delete_secret(
        application_id: Uuid,
        secret_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        use model::application_secret_bindings::Column::*;

        let binding = ApplicationSecretBindings::find()
            .filter(
                Condition::all()
                    .add(ApplicationId.eq(application_id))
                    .add(SecretId.eq(secret_id)),
            )
            .one(database)
            .await?
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("cannot found secret with id={secret_id}"),
                )
            })?;

        ApplicationSecretBindings::delete(binding.into_active_model())
            .exec(database)
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
