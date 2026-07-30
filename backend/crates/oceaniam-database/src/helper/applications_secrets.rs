use crate::error::Error;
use axum::http::StatusCode;
use chrono::Utc;
use oceaniam_vo::pagination::{PageParam, PagedResponse};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, TryInsertResult,
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
    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.create_secret_unbound",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_secret_unbound(
        id: Uuid,
        secret_prefix: impl Into<String> + Send,
        secret_verifier: Vec<u8>,
        hmac_key_version: i32,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_secrets::Model, Error> {
        Ok(model::application_secrets::Model {
            id,
            secret_prefix: secret_prefix.into(),
            secret_verifier,
            hmac_key_version,
            created_at: Utc::now().into(),
            revoked_at: None,
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.create_secret",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_secret(
        application_id: Uuid,
        id: Uuid,
        secret_prefix: impl Into<String> + Send,
        secret_verifier: Vec<u8>,
        hmac_key_version: i32,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::application_secrets::Model, Error> {
        let transaction = database.begin().await?;

        let model = model::application_secrets::Model {
            id,
            secret_prefix: secret_prefix.into(),
            secret_verifier,
            hmac_key_version,
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.get_secret_models",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.get_all_secrets_of",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.get_secret",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.get_application_ids_of_secret",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.get_application_ids_grouped_by_secret_ids",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_application_ids_grouped_by_secret_ids(
        secret_ids: Option<&[Uuid]>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<HashMap<Uuid, Vec<Uuid>>, Error> {
        use model::application_secret_bindings::Column::*;

        let mut query = ApplicationSecretBindings::find()
            .select_only()
            .column(SecretId)
            .column(ApplicationId);

        if let Some(ids) = secret_ids {
            if ids.is_empty() {
                return Ok(HashMap::new());
            }
            query = query.filter(SecretId.is_in(ids.iter().copied()));
        }

        let bindings = query.into_tuple::<(Uuid, Uuid)>().all(database).await?;

        let mut grouped = HashMap::<Uuid, Vec<Uuid>>::new();
        for (secret_id, application_id) in bindings {
            grouped.entry(secret_id).or_default().push(application_id);
        }

        Ok(grouped)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.find_active_secret_candidates",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn find_active_secret_candidates(
        secret_prefix: &str,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::application_secrets::Model>, Error> {
        use model::application_secrets::Column::*;

        Ok(ApplicationSecrets::find()
            .filter(SecretPrefix.eq(secret_prefix))
            .filter(RevokedAt.is_null())
            .order_by_asc(Id)
            .all(database)
            .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.upgrade_secret_verifier",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn upgrade_secret_verifier(
        secret_id: Uuid,
        old_hmac_key_version: i32,
        old_secret_verifier: &[u8],
        new_hmac_key_version: i32,
        new_secret_verifier: Vec<u8>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        use model::application_secrets::Column::*;

        let result = ApplicationSecrets::update_many()
            .col_expr(
                HmacKeyVersion,
                sea_orm::sea_query::Expr::value(new_hmac_key_version),
            )
            .col_expr(
                SecretVerifier,
                sea_orm::sea_query::Expr::value(new_secret_verifier),
            )
            .filter(Id.eq(secret_id))
            .filter(HmacKeyVersion.eq(old_hmac_key_version))
            .filter(SecretVerifier.eq(old_secret_verifier.to_vec()))
            .exec(database)
            .await?;

        Ok(result.rows_affected == 1)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.get_hmac_key_versions",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_hmac_key_versions(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<i32>, Error> {
        use model::application_secrets::Column::*;

        Ok(ApplicationSecrets::find()
            .select_only()
            .column(HmacKeyVersion)
            .filter(RevokedAt.is_null())
            .distinct()
            .into_tuple::<i32>()
            .all(database)
            .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.create_binding",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_binding(
        secret_id: Uuid,
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let result =
            ApplicationSecretBindings::insert(model::application_secret_bindings::ActiveModel {
                secret_id: sea_orm::ActiveValue::Set(secret_id),
                application_id: sea_orm::ActiveValue::Set(application_id),
            })
            .on_conflict_do_nothing()
            .exec(database)
            .await?;

        if matches!(result, TryInsertResult::Conflicted) {
            return Err(Error::with_code(
                StatusCode::CONFLICT,
                format!(
                    "binding between secret={secret_id} and application={application_id} already exists"
                ),
            ));
        }

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.delete_secret",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.delete_secret_by_id",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.get_all_secret_ids",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_all_secret_ids(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<Uuid>, Error> {
        use model::application_secrets::Column::*;

        Ok(all_secrets_base()
            .select_only()
            .column(Id)
            .distinct()
            .into_tuple::<Uuid>()
            .all(database)
            .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.applications_secrets.get_all_secret_models",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_all_secret_models(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::application_secrets::Model>, Error> {
        Ok(all_secrets_base().all(database).await?)
    }
}

fn all_secrets_base() -> sea_orm::Select<ApplicationSecrets> {
    ApplicationSecrets::find()
}

impl ApplicationSecretsHelper for ApplicationSecrets {}
