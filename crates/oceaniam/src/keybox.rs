use backon::{ExponentialBuilder, RetryableWithContext};
use im::HashMap;
use itertools::Itertools;
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::{SafeTransactionConnectionTrait, key_boxes::KeyBoxesModelHelper},
    model::{key_boxes::Model as Key, prelude::KeyBoxes},
};
use oceaniam_keybox::{KeyBox as InnerKeyBox, keybox::StatusMaskedKey};
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

    /// The [application_id](oceaniam_database::model::key_boxes::Model::id) associated with these keys.
    pub application_id: Uuid,
}

#[allow(unused)]
impl KeyBox {
    pub fn with_keys(application_id: Uuid, keys: HashMap<Uuid, Key>) -> Self {
        Self(InnerKeyBox::with_keys(application_id, keys))
    }

    /// Synchronizes the local key box with the database.
    ///
    /// This method performs a bidirectional synchronization:
    /// - Pushes new local keys to the database
    /// - Updates key statuses in the database
    /// - Pulls keys from the database to local storage
    ///
    /// Uses exponential backoff retry mechanism for resilience.
    pub async fn sync_database<C>(&mut self, database: &C) -> Result<HashMap<Uuid, Key>, Error>
    where
        C: SafeTransactionConnectionTrait,
    {
        let database = database.begin().await?;
        let keys = self.0.get_keys();
        let (ctx, result) = {
            |ctx: SyncRetryContext| async move {
                let r = Self::sync_func(&ctx).await;

                // Check the result to match if application has been removed
                match r {
                    Err(Error::Db(DbErr::RecordNotFound(_))) => (ctx, Ok(HashMap::default())),
                    _ => (ctx, r),
                }
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

    /// Core synchronization function that handles the actual database operations.
    ///
    /// Performs three main operations:
    /// 1. Syncs additions: Inserts new local keys to database that don't exist there
    /// 2. Syncs status: Updates key statuses for keys that need status changes
    /// 3. Syncs from database: Retrieves all keys for the application from database
    ///
    /// Returns a HashMap of all keys retrieved from the database.
    ///
    /// # Note
    ///
    /// This function does not check if application has been removed.
    ///
    /// _**SO INVOKER SHOULD CHECK THE ERROR**_.
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

    /// Adds a new key to the key box with the specified options.
    ///
    ///
    /// # Type Parameters
    /// * `T` - A type that can be converted into a StandloneKey
    ///
    /// # Parameters
    /// * `key` - The key to add to the key box
    /// * `option` - Configuration options for the key (expiry, activation, etc.)
    ///
    /// # Returns
    /// * `Ok(())` if the key was successfully added
    /// * `Err` if the key conversion failed
    pub fn put_key<T>(
        &mut self,
        key: T,
        option: oceaniam_keybox::keybox::KeyOption,
    ) -> Result<(), oceaniam_keybox::error::Error>
    where
        T: TryInto<oceaniam_keybox::keybox::StandloneKey, Error = oceaniam_keybox::error::Error>,
    {
        self.0.put_key(key, option)
    }

    /// Retrieves a key from the key box by its UUID.
    ///
    /// Returns a status-masked key that hides sensitive information about the key's status.
    /// Returns `None` if the key is not found.
    ///
    /// # Parameters
    /// * `key_id` - The UUID of the key to retrieve
    pub fn get_key(&self, key_id: &Uuid) -> Option<StatusMaskedKey> {
        self.0.get_key(key_id)
    }
}

impl Clone for KeyBox {
    fn clone(&self) -> Self {
        Self(self.0.deep_clone())
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub struct KeyBoxManager {
    applications: HashMap<Uuid, KeyBox>,
}

#[allow(unused)]
impl KeyBoxManager {
    /// Creates a new KeyBoxManager with all applications that have keys in the database.
    ///
    /// This method queries the database for all distinct application IDs that have associated key
    /// boxes, then creates KeyBox instances for each of them.
    ///
    /// # Parameters
    /// * `database` - The database connection trait
    ///
    /// # Returns
    /// * `Ok(KeyBoxManager)` with all applications loaded
    /// * `Err` if the database query fails
    pub async fn new(database: &impl ConnectionTrait) -> Result<Self, Error> {
        use oceaniam_database::model::key_boxes::Column::ApplicationId;
        use oceaniam_database::model::prelude::KeyBoxes;
        use sea_orm::QuerySelect;

        let application_ids: Vec<Uuid> = KeyBoxes::find()
            .select_only()
            .column(ApplicationId)
            .distinct()
            .all(database)
            .await?
            .into_iter()
            .map(|model| model.application_id)
            .collect();

        Self::with_applications_id(database, application_ids).await
    }

    /// Creates a KeyBoxManager for specific application IDs.
    ///
    /// This method queries the database for keys belonging to the specified
    /// application IDs and creates KeyBox instances for each application.
    ///
    /// # Parameters
    /// * `database` - The database connection trait
    /// * `application_ids` - An iterator of application UUIDs to load
    ///
    /// # Returns
    /// * `Ok(KeyBoxManager)` with the specified applications loaded
    /// * `Err` if the database query fails
    pub async fn with_applications_id(
        database: &impl ConnectionTrait,
        application_ids: impl IntoIterator<Item = Uuid>,
    ) -> Result<Self, Error> {
        use oceaniam_database::model::key_boxes::Column::ApplicationId;
        use oceaniam_database::model::prelude::KeyBoxes;

        let application_ids: Vec<Uuid> = application_ids.into_iter().collect();

        if application_ids.is_empty() {
            return Ok(Self::default());
        }

        let keys = KeyBoxes::find()
            .filter(ApplicationId.is_in(application_ids))
            .all(database)
            .await?;

        let applications: HashMap<Uuid, KeyBox> = keys
            .into_iter()
            .into_group_map_by(|key| key.application_id)
            .into_iter()
            .map(|(app_id, keys)| {
                let keys_map: HashMap<Uuid, Key> =
                    keys.into_iter().map(|key| (key.id, key)).collect();
                (app_id, KeyBox::with_keys(app_id, keys_map))
            })
            .collect();

        Ok(Self { applications })
    }

    /// Synchronizes a specific application's key box with the database.
    ///
    /// This method finds the key box for the given application ID and performs a database
    /// synchronization. If the application doesn't exist in the manager, this method returns Ok
    /// without doing anything.
    ///
    /// # Parameters
    /// * `application_id` - The UUID of the application whose key box should be synced
    /// * `database` - The database connection trait for transaction management
    ///
    /// # Returns
    /// * `Ok(())` on successful sync or if application not found
    /// * `Err` if the synchronization fails
    async fn sync_keybox_with_application_id<C>(
        &mut self,
        application_id: &Uuid,
        database: &C,
    ) -> Result<(), Error>
    where
        C: SafeTransactionConnectionTrait,
    {
        if let Some(keybox) = self.applications.get_mut(application_id) {
            let database = database.begin().await?;
            keybox.sync_database(&database).await?;
            database.commit().await?;
        }

        Ok(())
    }

    /// Retrieves a reference to a key box for the specified application.
    ///
    /// # Parameters
    /// * `application_id` - The UUID of the application whose key box to retrieve
    ///
    /// # Returns
    /// * `Some(&KeyBox)` if the application exists in the manager
    /// * `None` if the application is not found
    ///
    /// # Note
    ///
    /// You can clone the returns because clone action should be cheap.
    ///
    /// And the best way to using KeyBox is also get a deep clone for your own.
    async fn get_keybox(&self, application_id: &Uuid) -> Option<&KeyBox> {
        self.applications.get(application_id)
    }
}
