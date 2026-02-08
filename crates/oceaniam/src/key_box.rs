use backon::{ExponentialBuilder, RetryableWithContext};
use im::HashMap;
use oceaniam_common::error::Error;
use oceaniam_database::{helper::SafeTransactionConnectionTrait, model::key_boxes::Model as Key};
use oceaniam_keybox::KeyBox as InnerKeyBox;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

#[derive(Debug)]
pub struct KeyBox(InnerKeyBox);

impl KeyBox {
    pub async fn sync_database<C>(&mut self, database: C) -> Result<(), Error>
    where
        C: SafeTransactionConnectionTrait,
    {
        let database = database.begin().await?;
        let keys = self.0.get_keys();
        let ((database, _), _) = {
            |ctx: (DatabaseTransaction, HashMap<Uuid, Key>)| async move {
                let (database, keys) = ctx;
                let r = Self::func((&database, keys.clone())).await;

                ((database, keys), r)
            }
        }
        .retry(ExponentialBuilder::default())
        .context((database, keys))
        .await;
        database.commit().await?;

        Ok(())
    }

    async fn func(ctx: (&DatabaseTransaction, HashMap<Uuid, Key>)) -> Result<(), Error> {
        let (database, keys) = ctx;

        for (kid, key) in keys.iter() {}

        todo!()
    }
}
