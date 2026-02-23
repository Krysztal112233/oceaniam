use std::{sync::Arc, time::Duration};

use moka::future::{Cache, CacheBuilder};
use oceaniam_common::error::Error;
use oceaniam_database::{helper::revoked_jwts::RevokedJwtsHelper, model::prelude::RevokedJwts};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait};
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

    pub async fn is_revoked(&self, jti: impl Into<Uuid> + Copy) -> Result<bool, Arc<Error>> {
        self.status
            .try_get_with(jti.into(), async {
                match RevokedJwts::find_by_id(jti.into())
                    .one(&self.database)
                    .await
                {
                    Ok(record) => Ok(record.is_some()),
                    Err(DbErr::RecordNotFound(_)) => Ok(true),
                    Err(e) => Err(Error::from(e)),
                }
            })
            .await
    }

    pub async fn set_revoked(&self, jti: impl Into<Uuid> + Copy) -> Result<(), Error> {
        self.status.insert(jti.into(), true).await;

        RevokedJwts::revoke(jti.into(), &self.database).await?;

        Ok(())
    }
}
