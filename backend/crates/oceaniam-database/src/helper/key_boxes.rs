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
    /// System keys are identified by the constant `SYSTEM_APPLICATION_UUID`.
    ///
    /// # Arguments
    ///
    /// * `database` - A database connection that supports transactions
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of key models, or an error if the query fails.
    async fn get_system_keys(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::key_boxes::Model>, Error> {
        Self::get_application_keys(consts::SYSTEM_APPLICATION_UUID, database).await
    }

    /// Retrieves all keys for a given application from the database.
    async fn get_application_keys(
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::key_boxes::Model>, Error> {
        Ok(KeyBoxes::find()
            .filter(model::key_boxes::Column::ApplicationId.eq(application_id))
            .all(database)
            .await
            .inspect_err(|e| error!("{e}"))?)
    }

    /// Persists all in-memory keys to the database for a given application.
    ///
    /// Uses `INSERT ... ON CONFLICT (id) DO UPDATE` so that every row call
    /// is an atomic upsert — existing rows are updated, new rows are inserted.
    /// No explicit transaction wrapper is needed because the single statement
    /// provides statement-level atomicity.
    async fn update_application_keys(
        application_id: Uuid,
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
                        model::key_boxes::Column::ApplicationId,
                    ])
                    .to_owned(),
            )
            .exec_with_returning_keys(database)
            .await
            .inspect_err(|e| error!("{e}"))?;

        debug!("updated application({application_id}) keys {updated:?}");

        Ok(())
    }
}

impl KeyBoxesHelper for KeyBoxes {}
