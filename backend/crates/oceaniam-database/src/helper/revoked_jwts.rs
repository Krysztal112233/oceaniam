use chrono::Utc;
use oceaniam_common::error::Error;
use sea_orm::{
    ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, QueryOrder, QuerySelect, StreamTrait,
};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{
        self,
        prelude::RevokedJwts,
        revoked_jwts::{self, Model},
    },
};

#[async_trait::async_trait]
pub trait RevokedJwtsHelper {
    async fn revoke(
        jti: impl Into<Uuid> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::revoked_jwts::Model, Error> {
        Ok(model::revoked_jwts::Model {
            jti: jti.into(),
            revoked_at: Utc::now().into(),
        }
        .into_active_model()
        .insert(database)
        .await?)
    }

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
}

impl RevokedJwtsHelper for RevokedJwts {}
