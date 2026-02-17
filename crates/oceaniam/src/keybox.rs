use std::time::Duration;

use axum::http::StatusCode;
use log::error;
use moka::future::{Cache, CacheBuilder};
use oceaniam_common::{error::Error, jwks::JwkSet};
use oceaniam_database::{helper::key_boxes::KeyBoxesHelper, model::prelude::KeyBoxes};
use oceaniam_keybox::KeyBox;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApplicationKeyBoxManager {
    database: DatabaseConnection,
    boxes: Cache<Uuid, KeyBox>,
    banned: Cache<Uuid, ()>,
    jwks: Cache<Uuid, JwkSet>,
}

#[allow(unused)]
impl ApplicationKeyBoxManager {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            boxes: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
            banned: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
            jwks: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
        }
    }

    pub async fn get_keybox(&mut self, application_id: Uuid) -> Option<KeyBox> {
        let database = self.database.clone();

        self.boxes
            .try_get_with::<_, Error>(application_id, async {
                if self.banned.contains_key(&application_id) {
                    return Err(Error::with_code(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("cannot get keybox of `{application_id}`"),
                    ));
                };

                let keys = KeyBoxes::get_application_keys(application_id, &database)
                    .await
                    .inspect_err(|e| error!("{e}"))?
                    .into_iter()
                    .map(|it| (it.id, it))
                    .collect();

                let keybox = KeyBox::with_keys(application_id, keys);

                if keybox.get_keys().is_empty() {
                    self.banned.insert(application_id, ()).await;

                    Err(Error::with_code(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("cannot get keybox of `{application_id}`"),
                    ))
                } else {
                    Ok(keybox)
                }
            })
            .await
            .ok()
    }

    pub async fn get_jwks(&mut self, application_id: Uuid) -> Option<JwkSet> {
        self.jwks
            .optionally_get_with(application_id, async {
                Some(JwkSet::from(self.clone().get_keybox(application_id).await?))
            })
            .await
    }
}
