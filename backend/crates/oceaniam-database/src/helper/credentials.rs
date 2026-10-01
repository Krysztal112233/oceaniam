use sea_orm::{EntityTrait, IntoActiveModel, sea_query::OnConflict};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Credentials},
};

#[async_trait::async_trait]
pub trait CredentialsHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.credentials.get_credential_by_id",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_credential_by_id(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Option<model::credentials::Model>, Error> {
        Ok(Credentials::find_by_id(id).one(database).await?)
    }

    /// Deletes a credential by id, reporting a not-found error when no row was removed.
    #[tracing::instrument(
        level = "info",
        name = "db.credentials.delete_credential_by_id",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn delete_credential_by_id(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let result = Credentials::delete_by_id(id).exec(database).await?;
        if result.rows_affected != 1 {
            return Err(Error::not_found(format!("credential id={id} not found")));
        }

        Ok(())
    }

    /// Atomically inserts or replaces a credential row.
    ///
    /// Uses `INSERT ... ON CONFLICT (id) DO UPDATE` so concurrent upserts for the same id
    /// resolve at statement level instead of racing a read-then-write into a primary-key
    /// collision (see the `key_boxes` upsert pattern).
    #[tracing::instrument(
        level = "info",
        name = "db.credentials.upsert_credential",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn upsert_credential(
        id: Uuid,
        phc: String,
        totp: Option<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::credentials::Model, Error> {
        use model::credentials::Column;

        let active_model = model::credentials::Model { id, phc, totp }.into_active_model();

        Ok(Credentials::insert(active_model)
            .on_conflict(
                OnConflict::column(Column::Id)
                    .update_columns([Column::Phc, Column::Totp])
                    .to_owned(),
            )
            .exec_with_returning(database)
            .await?)
    }
}

impl CredentialsHelper for Credentials {}
