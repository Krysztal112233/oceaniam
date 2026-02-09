use backon::{ExponentialBuilder, RetryableWithContext};
use im::HashMap;
use itertools::Itertools;
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::{SafeTransactionConnectionTrait, key_boxes::KeyBoxesModelHelper},
    model::{key_boxes::Model as Key, prelude::KeyBoxes},
};
use oceaniam_keybox::{KeyBox as InnerKeyBox, key_box::StatusMaskedKey};
use sea_orm::{DatabaseTransaction, FromQueryResult, IntoActiveModel, prelude::*};
use uuid::Uuid;

#[derive(Debug)]
pub struct KeyBox(InnerKeyBox);

/// Context structure for the synchronization retry operation.
///
/// This struct holds all necessary context data for retrying the sync operation, including the
/// database transaction, local keys, and
/// [application_id](oceaniam_database::model::key_boxes::Model::id)
#[derive(Debug)]
struct SyncRetryContext {
    /// The database transaction for executing database operations.
    pub database: DatabaseTransaction,

    /// Local keys that need to be synchronized with the database.
    pub local_keys: HashMap<Uuid, Key>,

    /// The application ID associated with these keys.
    pub application_id: Uuid,
}

#[allow(unused)]
impl KeyBox {
    pub async fn sync_database<C>(&mut self, database: C) -> Result<HashMap<Uuid, Key>, Error>
    where
        C: SafeTransactionConnectionTrait,
    {
        let database = database.begin().await?;
        let keys = self.0.get_keys();
        let (ctx, result) = {
            |ctx: SyncRetryContext| async move {
                let r = Self::sync_func(&ctx).await;

                (ctx, r)
            }
        }
        .retry(ExponentialBuilder::default())
        .context(SyncRetryContext {
            database,
            local_keys: keys,
            application_id: self.0.application_id(),
        })
        .await;
        ctx.database.commit().await?;

        let db_keys = result?;
        self.0.sync(db_keys.clone());
        Ok(db_keys)
    }

    async fn sync_func(ctx: &SyncRetryContext) -> Result<HashMap<Uuid, Key>, Error> {
        let SyncRetryContext {
            database,
            local_keys,
            application_id,
        } = ctx;

        /////////////////////////////////////////////////////
        //            sync addition to database            //
        /////////////////////////////////////////////////////
        {
            use oceaniam_database::model::key_boxes::{Column::*, Model as KeyBoxModel};

            #[derive(DerivePartialModel, FromQueryResult)]
            #[sea_orm(entity = "oceaniam_database::model::key_boxes::Entity")]
            struct PartialKeyBox {
                id: Uuid,
            }

            let key_ids = local_keys.keys().cloned().collect_vec();
            let updated_keys = KeyBoxes::find()
                .filter(Id.is_not_in(key_ids))
                .into_partial_model::<PartialKeyBox>()
                .all(database)
                .await?
                .into_iter()
                .map(|PartialKeyBox { id }| id)
                .flat_map(|it| local_keys.get(&it))
                .cloned()
                .collect_vec();

            KeyBoxes::insert_many(updated_keys.into_iter().map(KeyBoxModel::into_active_model))
                .exec(database)
                .await?;
        }

        /////////////////////////////////////////////////////
        //              sync status to database            //
        /////////////////////////////////////////////////////
        {
            use oceaniam_database::model::key_boxes::Model as KeyBoxModel;

            let updated_keys = local_keys
                .values()
                .cloned()
                .flat_map(|it| {
                    it.should_update_status()
                        .map(|status| KeyBoxModel { status, ..it })
                })
                .collect_vec();

            KeyBoxes::insert_many(updated_keys.into_iter().map(KeyBoxModel::into_active_model))
                .exec(database)
                .await?;
        }

        /////////////////////////////////////////////////////
        //                   sync from database            //
        /////////////////////////////////////////////////////
        let updated = {
            use oceaniam_database::model::key_boxes::{Column::*, Model as KeyBoxModel};

            KeyBoxes::find()
                .filter(ApplicationId.eq(*application_id))
                .all(database)
                .await?
        };

        Ok(updated.into_iter().map(|it| (it.id, it)).collect::<_>())
    }

    pub fn put_key<T>(
        &mut self,
        key: T,
        option: oceaniam_keybox::key_box::KeyOption,
    ) -> Result<(), oceaniam_keybox::error::Error>
    where
        T: TryInto<oceaniam_keybox::key_box::StandloneKey, Error = oceaniam_keybox::error::Error>,
    {
        self.0.put_key(key, option)
    }

    pub fn get_key(&self, key_id: &Uuid) -> Option<StatusMaskedKey> {
        self.0.get_key(key_id)
    }
}
