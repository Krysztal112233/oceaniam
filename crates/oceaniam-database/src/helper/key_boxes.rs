use log::error;
use oceaniam_common::error::Error;
use sea_orm::EntityTrait;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::KeyBoxes},
};

#[async_trait::async_trait]
pub trait KeyBoxesHelper {
    async fn get_system_key_boxes(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        // Well, this equals "00000000-0000-0000-0000-000000000000".
        let resu = KeyBoxes::find_by_id(uuid::Uuid::default())
            .one(database)
            .await
            .inspect_err(|e| error!("{e}"))?;

        Ok(())
    }
}

impl KeyBoxesHelper for KeyBoxes {}
