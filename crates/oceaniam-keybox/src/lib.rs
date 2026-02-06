pub mod error;
pub mod key;

use std::collections::HashMap;

use oceaniam_database::model::key_boxes::Model as KeyBoxModel;
use uuid::Uuid;

pub use oceaniam_database::model::sea_orm_active_enums::KeyStatus;

/// [KeyBox] is used to manage multiple keys, providing expiration checking and key management functionality
#[derive(Debug, Clone)]
pub struct KeyBox {
    /// Stores all keys with [KeyBoxModel::key_id] as the key
    keys: HashMap<Uuid, KeyBoxModel>,
}

impl KeyBox {
    /// Creates a new empty [KeyBox]
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Adds a new key
    pub fn insert_key(&mut self, key: impl Into<KeyBoxModel>) {
        let key = key.into();
        self.keys.insert(key.key_id, key);
    }

    /// Gets a key by key_id
    pub fn get_key(&self, key_id: &Uuid) -> Option<&KeyBoxModel> {
        self.keys.get(key_id)
    }

    /// Gets all keys
    pub fn get_all_keys(&self) -> Vec<&KeyBoxModel> {
        self.keys.values().collect()
    }

    /// Removes the specified key
    pub fn remove_key(&mut self, key_id: &Uuid) -> Option<KeyBoxModel> {
        self.keys.remove(key_id)
    }
}

impl Default for KeyBox {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<KeyBoxModel>> for KeyBox {
    fn from(models: Vec<KeyBoxModel>) -> Self {
        let keys = models
            .into_iter()
            .map(|model| (model.key_id, model))
            .collect();
        KeyBox { keys }
    }
}
