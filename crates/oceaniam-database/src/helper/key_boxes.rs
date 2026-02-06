use log::error;
use oceaniam_common::{consts, error::Error};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::KeyBoxes},
};

#[async_trait::async_trait]
pub trait KeyBoxesHelper {
    async fn get_system_keys(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<model::key_boxes::Model>, Error> {
        Ok(KeyBoxes::find()
            .filter(model::key_boxes::Column::ApplicationId.eq(consts::SYSTEM_APPLICATION_UUID))
            .all(database)
            .await
            .inspect_err(|e| error!("{e}"))?)
    }
}

impl KeyBoxesHelper for KeyBoxes {}
