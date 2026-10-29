//! Helper functions and traits for key_boxes operations.

use crate::error::Error;
use oceaniam_common::consts;
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Statement, sea_query::OnConflict,
};
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

    /// Serializes worker rotations for a tenant until the caller's transaction ends.
    ///
    /// The schema is part of the lock namespace so isolated databases/schemas do not contend.
    /// Call before loading keys, then persist and commit on the same transaction.
    #[tracing::instrument(
        level = "info",
        name = "db.key_boxes.lock_rotation",
        skip_all,
        fields(otel.kind = "internal", %tenant_id)
    )]
    async fn lock_rotation(
        tenant_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        database.query_one_raw(Statement::from_sql_and_values(
            database.get_database_backend(),
            "SELECT pg_advisory_xact_lock(hashtextextended(current_schema() || ':key_rotation:' || ($1::uuid)::text, 0))",
            [tenant_id.into()],
        )).await?;
        Ok(())
    }

    /// Counts all persisted key box rows.
    ///
    /// Supports the startup KEK health probe in `oceaniam::app`, which skips verification on
    /// fresh installs (empty `key_boxes`).
    #[tracing::instrument(
        level = "info",
        name = "db.key_boxes.count",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn count_key_boxes(database: &impl SafeTransactionConnectionTrait) -> Result<u64, Error> {
        Ok(KeyBoxes::find()
            .count(database)
            .await
            .inspect_err(|e| error!("{e}"))?)
    }

    /// Returns one arbitrary persisted key box row.
    ///
    /// Supports the startup KEK health probe in `oceaniam::app`, which decrypts a single stored
    /// secret to verify the configured master key against the one used during migration.
    #[tracing::instrument(
        level = "info",
        name = "db.key_boxes.get_any",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_any_key_box(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Option<model::key_boxes::Model>, Error> {
        KeyBoxes::find()
            .one(database)
            .await
            .inspect_err(|e| error!("{e}"))
            .map_err(Into::into)
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
