use crate::error::Error;
use axum::http::StatusCode;
use chrono::Utc;
use oceaniam_api::{PageInfo, PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QuerySelect,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::model::prelude::{ApplicationSecretBindings, ApplicationSecrets};
use crate::{
    helper::{PagedExecutor, PagedSelect, SafeTransactionConnectionTrait},
    model::{self},
};

#[async_trait::async_trait]
pub trait ApplicationSecretsHelper {
    async fn create_secret_unbound(
        id: Uuid,
        secret: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_secrets::Model, Error> {
        Ok(model::application_secrets::Model {
            id,
            secret: secret.into(),
            created_at: Utc::now().into(),
            revoked_at: None,
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

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

    async fn get_secrets_of(
        application_id: Uuid,
        paged: Option<PageParam>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::application_secrets::Model>, Error> {
        match paged {
            Some(paged) => {
                let all_items = Self::get_all_secrets_of(application_id, database).await?;
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
            None => Ok(PagedResponse::with_entire(
                Self::get_all_secrets_of(application_id, database).await?,
            )),
        }
    }

    async fn get_secret_models(
        page: PageParam,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<PagedResponse<model::application_secrets::Model>, Error> {
        ApplicationSecrets::find()
            .paged(page)
            .paginate(database, page.per_page)
            .fetch_paged(page)
            .await
    }

    async fn get_all_secrets_of(
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

    async fn get_secret(
        secret_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_secrets::Model, Error> {
        ApplicationSecrets::find_by_id(secret_id)
            .one(database)
            .await?
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("cannot found secret with id={secret_id}"),
                )
            })
    }

    async fn get_application_ids_of_secret(
        secret_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<Uuid>, Error> {
        use model::application_secret_bindings::Column::*;

        Ok(ApplicationSecretBindings::find()
            .filter(SecretId.eq(secret_id))
            .select_only()
            .column(ApplicationId)
            .into_tuple::<Uuid>()
            .all(database)
            .await?)
    }

    async fn get_all_application_ids_grouped_by_secret_id(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<HashMap<Uuid, Vec<Uuid>>, Error> {
        use model::application_secret_bindings::Column::*;

        let bindings = ApplicationSecretBindings::find()
            .select_only()
            .column(SecretId)
            .column(ApplicationId)
            .into_tuple::<(Uuid, Uuid)>()
            .all(database)
            .await?;

        let mut grouped = HashMap::<Uuid, Vec<Uuid>>::new();
        for (secret_id, application_id) in bindings {
            grouped.entry(secret_id).or_default().push(application_id);
        }

        Ok(grouped)
    }

    async fn get_application_ids_grouped_by_secret_ids(
        secret_ids: Vec<Uuid>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<HashMap<Uuid, Vec<Uuid>>, Error> {
        use model::application_secret_bindings::Column::*;

        if secret_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let bindings = ApplicationSecretBindings::find()
            .filter(SecretId.is_in(secret_ids))
            .select_only()
            .column(SecretId)
            .column(ApplicationId)
            .into_tuple::<(Uuid, Uuid)>()
            .all(database)
            .await?;

        let mut grouped = HashMap::<Uuid, Vec<Uuid>>::new();
        for (secret_id, application_id) in bindings {
            grouped.entry(secret_id).or_default().push(application_id);
        }

        Ok(grouped)
    }

    async fn find_secret_can_be_used_for(
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

    async fn delete_secret_by_id(
        secret_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let secret = Self::get_secret(secret_id, database).await?;

        ApplicationSecrets::delete(secret.into_active_model())
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

    async fn get_all_secret_models(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::application_secrets::Model>, Error> {
        Ok(ApplicationSecrets::find().all(database).await?)
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
