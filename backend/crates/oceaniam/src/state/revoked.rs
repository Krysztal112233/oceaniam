use std::{sync::Arc, time::Duration};

use crate::error::Error;
use moka::future::{Cache, CacheBuilder};
use oceaniam_database::{helper::revoked_jwts::RevokedJwtsHelper, model::prelude::RevokedJwts};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RevokedJwt {
    database: DatabaseConnection,
    status: Cache<Uuid, bool>,
}

impl RevokedJwt {
    pub fn new(database: DatabaseConnection) -> Self {
        let cache = CacheBuilder::default()
            .max_capacity(102400)
            .time_to_live(Duration::from_mins(5))
            .build();
        Self {
            database,
            status: cache,
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "jwt_revocation.is_revoked",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn is_revoked(&self, jti: impl Into<Uuid> + Copy) -> Result<bool, Arc<Error>> {
        self.status
            .try_get_with(jti.into(), async {
                RevokedJwts::is_revoked(jti.into(), &self.database)
                    .await
                    .map_err(Into::into)
            })
            .await
    }

    #[tracing::instrument(
        level = "info",
        name = "jwt_revocation.set_revoked",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn set_revoked(&self, jti: impl Into<Uuid> + Copy) -> Result<(), Error> {
        self.status.insert(jti.into(), true).await;

        RevokedJwts::revoke(jti.into(), &self.database).await?;

        Ok(())
    }
}
