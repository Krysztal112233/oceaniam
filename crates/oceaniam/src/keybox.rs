use std::time::Duration;

use moka::future::{Cache, CacheBuilder};
use oceaniam_common::error::Error;
use oceaniam_database::{helper::key_boxes::KeyBoxesHelper, model::prelude::KeyBoxes};
use oceaniam_keybox::KeyBox;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct KeyBoxManager {
    database: DatabaseConnection,
    boxes: Cache<Uuid, KeyBox>,
}

#[allow(unused)]
impl KeyBoxManager {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            boxes: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
        }
    }

    pub async fn get_keybox(&mut self, application_id: Uuid) -> Option<KeyBox> {
        let database = self.database.clone();

        self.boxes
            .try_get_with::<_, Error>(application_id, async {
                let keys = KeyBoxes::get_application_keys(application_id, &database)
                    .await?
                    .into_iter()
                    .map(|it| (it.id, it))
                    .collect();

                Ok(KeyBox::with_keys(application_id, keys))
            })
            .await
            .ok()
    }
}
