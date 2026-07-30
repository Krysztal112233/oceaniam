//! Helper functions and traits for key_boxes operations.

use crate::error::Error;
use oceaniam_common::consts;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::OnConflict};
use tracing::{debug, error};
use uuid::Uuid;

use crate::{helper::SafeTransactionConnectionTrait, model, model::prelude::KeyBoxes};

/// Helper trait providing database query methods for key boxes.
#[async_trait::async_trait]
pub trait KeyBoxesHelper {
    /// Retrieves all system-level keys from the database.
    ///
    /// System keys are identified by the constant `SYSTEM_TENANT_UUID`.
    ///
    /// # Arguments
    ///
    /// * `database` - A database connection that supports transactions
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of key models, or an error if the query fails.
    #[tracing::instrument(
        level = "info",
        name = "db.key_boxes.get_system_keys",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_system_keys(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::key_boxes::Model>, Error> {
        Self::get_tenant_keys(consts::SYSTEM_TENANT_UUID, database).await
    }

    /// Retrieves all keys for a given tenant from the database.
    #[tracing::instrument(
        level = "info",
        name = "db.key_boxes.get_tenant_keys",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_tenant_keys(
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::key_boxes::Model>, Error> {
        Ok(KeyBoxes::find()
            .filter(model::key_boxes::Column::TenantId.eq(tenant_id))
            .all(database)
            .await
            .inspect_err(|e| error!("{e}"))?)
    }

    /// Persists all in-memory keys to the database for a given tenant.
    ///
    /// Uses `INSERT ... ON CONFLICT (id) DO UPDATE` so that every row call
    /// is an atomic upsert — existing rows are updated, new rows are inserted.
    /// No explicit transaction wrapper is needed because the single statement
    /// provides statement-level atomicity.
    #[tracing::instrument(
        level = "info",
        name = "db.key_boxes.update_application_keys",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn update_application_keys(
        tenant_id: Uuid,
        keys: impl IntoIterator<Item = model::key_boxes::ActiveModel> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let updated = KeyBoxes::insert_many(keys)
            .on_conflict(
                OnConflict::column(model::key_boxes::Column::Id)
                    .update_columns([
                        model::key_boxes::Column::KeyAlg,
                        model::key_boxes::Column::Status,
                        model::key_boxes::Column::CreatedAt,
                        model::key_boxes::Column::ActivatedAt,
                        model::key_boxes::Column::RetiredAt,
                        model::key_boxes::Column::RevokedAt,
                        model::key_boxes::Column::ExpiresAt,
                        model::key_boxes::Column::Secret,
                        model::key_boxes::Column::TenantId,
                    ])
                    .to_owned(),
            )
            .exec_with_returning_keys(database)
            .await
            .inspect_err(|e| error!("{e}"))?;

        debug!("updated tenant({tenant_id}) keys {updated:?}");

        Ok(())
    }
}

impl KeyBoxesHelper for KeyBoxes {}
