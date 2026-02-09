//! Helper functions and traits for key_boxes operations.

use chrono::Utc;
use log::error;
use oceaniam_common::{consts, error::Error};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{
        self, key_boxes::Model as KeyBoxesModel, prelude::KeyBoxes, sea_orm_active_enums::KeyStatus,
    },
};

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
        Ok(KeyBoxes::find()
            .filter(model::key_boxes::Column::ApplicationId.eq(consts::SYSTEM_APPLICATION_UUID))
            .all(database)
            .await
            .inspect_err(|e| error!("{e}"))?)
    }

    /// Updates the status of this key based on its temporal properties.
    ///
    /// This is a marker method for the trait. The actual implementation
    /// is provided by `KeyBoxesModelHelper`.
    fn update_status(self) -> Self;
}

/// Helper trait for status management of key box models.
///
/// This trait provides methods to automatically determine and update
/// the status of a key based on its activation, retirement, and expiration times.
pub trait KeyBoxesModelHelper {
    /// Returns a new model with an updated status if necessary.
    ///
    /// The status is recalculated based on current time and the key's
    /// temporal properties (`activated_at`, `retired_at`, `expires_at`).
    ///
    /// # Returns
    ///
    /// A new `KeyBoxesModel` with potentially updated status.
    fn update_status(self) -> Self;

    /// Determines if the key's status should be updated based on current time.
    ///
    /// This method checks the key's temporal properties against the current time
    /// to determine the appropriate status (Active, Pending, or Retired).
    ///
    /// # Returns
    ///
    /// - `Some(KeyStatus)` if the status needs to be updated to the given value.
    /// - `None` if the current status is already correct.
    fn should_update_status(&self) -> Option<KeyStatus>;
}

impl KeyBoxesModelHelper for KeyBoxesModel {
    fn update_status(self) -> Self {
        let status = {
            match self.should_update_status() {
                Some(status) => status,
                None => self.status,
            }
        };

        KeyBoxesModel { status, ..self }
    }

    fn should_update_status(&self) -> Option<KeyStatus> {
        let KeyBoxesModel {
            activated_at,
            retired_at,
            expires_at,
            ..
        } = self;

        let status = {
            let now = Utc::now();

            if expires_at.is_some_and(|t| now >= t) || retired_at.is_some_and(|t| now >= t) {
                KeyStatus::Retired
            } else if activated_at.is_none_or(|t| now >= t) {
                KeyStatus::Active
            } else {
                KeyStatus::Pending
            }
        };

        if self.status != status {
            None
        } else {
            Some(status)
        }
    }
}
