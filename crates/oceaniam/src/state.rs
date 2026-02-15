use std::sync::Arc;

use axum::extract::FromRef;
use im::HashMap;
use log::{error, info, warn};
use oceaniam_common::{consts, error::Error, jwks::ManagedJwkSet};
use oceaniam_database::{
    helper::key_boxes::KeyBoxesHelper,
    model::{prelude::KeyBoxes, sea_orm_active_enums::KeyAlg},
};
use oceaniam_keybox::{KeyBox, key::rsa_key::RsaKey};
use parking_lot::RwLock;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{keybox::ApplicationKeyBoxManager, roller::BuiltinScheduledJwkSetRoller};

#[derive(Debug, Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
    pub application_keybox_manager: ApplicationKeyBoxManager,
    pub system_jwks: ManagedJwkSet,
    pub system_keybox: Arc<RwLock<KeyBox>>,
    pub _unit: (),
}

impl AppState {
    pub async fn new(database: DatabaseConnection) -> Result<Self, Error> {
        let keybox = ApplicationKeyBoxManager::new(database.clone());

        Ok(Self {
            database: database.clone(),
            application_keybox_manager: keybox,
            system_keybox: initial_system_keybox(database.clone()).await?,
            system_jwks: initial_system_jwks(database.clone()).await?,
            _unit: (),
        })
    }
}

async fn initial_system_jwks(database: DatabaseConnection) -> Result<ManagedJwkSet, Error> {
    let roller = BuiltinScheduledJwkSetRoller::new(database);
    let system_jwks = ManagedJwkSet::with_roller(roller).await?;

    if system_jwks.jwks().keys.is_empty() {
        warn!("could not find any jwks, the system may not be functioning correctly.")
    }

    Ok(system_jwks)
}

async fn initial_system_keybox(database: DatabaseConnection) -> Result<Arc<RwLock<KeyBox>>, Error> {
    let keys: HashMap<_, _> = KeyBoxes::get_system_keys(&database)
        .await?
        .into_iter()
        .map(|it| (it.id, it))
        .collect();

    let mut keybox = KeyBox::with_keys(consts::SYSTEM_APPLICATION_UUID, keys.clone());

    if keys.is_empty() {
        info!("could not find any system keys. a new system key will be generated.");

        let key = RsaKey::new(Uuid::now_v7(), KeyAlg::Rs512);
        keybox.put_key(key).inspect_err(|e| error!("{e}"))?;

        info!("the system key has been generated and is about to be written to the database.");

        keybox.clone().write_to(&database).await?;

        info!("system key has been written to database!");
    }

    Ok(Arc::new(RwLock::new(keybox)))
}

impl FromRef<AppState> for () {
    fn from_ref(input: &AppState) -> Self {
        input._unit
    }
}
