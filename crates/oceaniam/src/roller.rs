use std::time::Duration;

use log::error;
use oceaniam_common::{
    consts,
    error::Error,
    jwks::{JwkSet, ManagedJwkSet, roller::ManagedJwkSetRoller},
};
use oceaniam_database::{helper::key_boxes::KeyBoxesHelper, model::prelude::KeyBoxes};
use oceaniam_keybox::KeyBox;
use sea_orm::DatabaseConnection;

#[derive(Debug)]
pub struct BuiltinScheduledJwkSetRoller {
    database: DatabaseConnection,
}

impl BuiltinScheduledJwkSetRoller {
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }
}

#[async_trait::async_trait]
impl ManagedJwkSetRoller for BuiltinScheduledJwkSetRoller {
    async fn roll(&self, mut copy: ManagedJwkSet) -> Result<(), Error> {
        let copy = copy.clone();
        let cloned = copy.clone();
        let database = self.database.clone();

        tokio::spawn(async move {
            loop {
                // TODO: make this behavior configurable
                tokio::time::sleep(Duration::from_secs(5)).await;

                let _ = BuiltinOneShotJwkSetRoller::new(database.clone())
                    .roll(cloned.clone())
                    .await;
            }
        });

        BuiltinOneShotJwkSetRoller::new(self.database.clone())
            .roll(copy.clone())
            .await
    }
}

#[derive(Debug)]
pub struct BuiltinOneShotJwkSetRoller {
    database: DatabaseConnection,
}

impl BuiltinOneShotJwkSetRoller {
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }

    pub async fn pull(&self) -> Result<JwkSet, Error> {
        let keys = KeyBoxes::get_system_keys(&self.database)
            .await
            .inspect_err(|e| error!("{e}"))?
            .into_iter()
            .map(|it| (it.id, it))
            .collect();

        Ok(JwkSet::from(KeyBox::with_keys(
            consts::SYSTEM_APPLICATION_UUID,
            keys,
        )))
    }
}

#[async_trait::async_trait]
impl ManagedJwkSetRoller for BuiltinOneShotJwkSetRoller {
    async fn roll(&self, mut copy: ManagedJwkSet) -> Result<(), Error> {
        copy.set_jwks(self.pull().await?);

        Ok(())
    }
}
