use crate::error::Error;
use sea_orm::{
    ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, QueryOrder, QuerySelect, StreamTrait,
};
use uuid::Uuid;

use crate::{
    helper::{SafeTransactionConnectionTrait, db_clock_now},
    model::{
        self,
        prelude::RevokedJwts,
        revoked_jwts::{self, Model},
    },
};

#[async_trait::async_trait]
pub trait RevokedJwtsHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.revoked_jwts.revoke",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn revoke(
        jti: impl Into<Uuid> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::revoked_jwts::Model, Error> {
        Ok(model::revoked_jwts::Model {
            jti: jti.into(),
            revoked_at: db_clock_now(database).await?,
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.revoked_jwts.stream_recent",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn stream_recent<C>(
        database: &C,
    ) -> Result<impl futures::stream::Stream<Item = Result<Model, DbErr>>, Error>
    where
        C: SafeTransactionConnectionTrait + StreamTrait,
    {
        Ok(RevokedJwts::find()
            .order_by_desc(revoked_jwts::Column::RevokedAt)
            .limit(1024)
            .stream(database)
            .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "db.revoked_jwts.is_revoked",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn is_revoked(
        jti: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        Ok(RevokedJwts::find_by_id(jti).one(database).await?.is_some())
    }
}

impl RevokedJwtsHelper for RevokedJwts {}
