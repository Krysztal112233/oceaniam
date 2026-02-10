use chrono::{DateTime, FixedOffset, Utc};
use im::HashMap;
use oceaniam_database::model::{key_boxes::Model as Key, sea_orm_active_enums::KeyStatus};
use parking_lot::RwLock;
use serde_json::Value;
use uuid::Uuid;

use crate::{error::Error, key_alg::KeyAlg};

#[derive(Debug)]
pub struct StandloneKey {
    pub id: Uuid,
    pub key_alg: KeyAlg,
    pub secret: Value,
}

#[derive(Debug, Default)]
pub struct KeyOption {
    pub created_at: DateTime<FixedOffset>,
    pub activated_at: Option<DateTime<FixedOffset>>,
    pub retired_at: Option<DateTime<FixedOffset>>,
    pub expires_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug)]
pub enum StatusMaskedKey {
    Active(Key),
    Pending(Key),
    Retired(Key),

    /// If [Key] has been [KeyStatus::Revoked], **YOU CANNOT GET IT ANYWAY**.
    Revoked,
}

impl From<Key> for StatusMaskedKey {
    fn from(value: Key) -> Self {
        match value.status {
            KeyStatus::Active => Self::Active(value),
            KeyStatus::Pending => Self::Pending(value),
            KeyStatus::Retired => Self::Retired(value),
            KeyStatus::Revoked => Self::Revoked,
        }
    }
}

/// [KeyBox] is used to manage multiple keys, providing expiration checking and key management functionality
#[derive(Debug)]
pub struct KeyBox {
    /// Belong to application
    application_id: Uuid,

    /// Stores all keys with [Key::id] as the key
    keys: RwLock<HashMap<Uuid, Key>>,
}

impl KeyBox {
    pub fn new(application_id: Uuid) -> Self {
        Self {
            application_id,
            keys: RwLock::new(HashMap::new()),
        }
    }

    /// Adds a new key
    pub fn put_key<T>(
        &mut self,
        key: T,
        KeyOption {
            created_at,
            activated_at,
            retired_at,
            expires_at,
        }: KeyOption,
    ) -> Result<(), Error>
    where
        T: TryInto<StandloneKey, Error = Error>,
    {
        let StandloneKey {
            id,
            key_alg,
            secret,
        } = key.try_into()?;

        let status = {
            let now = Utc::now();

            if expires_at.is_some_and(|t| now >= t) || retired_at.is_some_and(|t| now >= t) {
                KeyStatus::Retired
            } else if activated_at.is_none_or(|t| now >= t) {
                KeyStatus::Active
            } else {
                KeyStatus::Pending
            }
        };

        let key = Key {
            id,
            key_alg: key_alg.into(),
            status,
            created_at,
            activated_at,
            retired_at,
            revoked_at: None,
            expires_at,
            secret,
            application_id: self.application_id,
        };

        self.keys.write().insert(id, key);
        Ok(())
    }

    /// Gets a key by key_id
    ///
    /// # Safety
    ///
    /// This function doesn't check the key is expired.
    pub unsafe fn get_key_unsafe(&self, key_id: &Uuid) -> Option<Key> {
        self.keys.read().get(key_id).cloned()
    }

    pub fn get_key(&self, key_id: &Uuid) -> Option<StatusMaskedKey> {
        unsafe { self.get_key_unsafe(key_id).map(StatusMaskedKey::from) }
    }

    /// Removes the specified key
    pub fn remove_key(&mut self, key_id: &Uuid) -> Option<Key> {
        self.keys.write().remove(key_id)
    }

    pub fn get_keys(&self) -> HashMap<Uuid, Key> {
        self.keys.read().clone()
    }

    pub fn sync(&mut self, income: HashMap<Uuid, Key>) {
        *self.keys.write() = income
    }

    pub fn application_id(&self) -> Uuid {
        self.application_id
    }

    pub fn deep_clone(&self) -> Self {
        Self {
            application_id: self.application_id,
            keys: RwLock::new(self.get_keys()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key::rsa_key::RsaKey;

    use super::*;
    use chrono::Duration;
    use oceaniam_database::model::sea_orm_active_enums::KeyAlg as InnerKeyAlg;

    fn create_rsa_key() -> RsaKey {
        RsaKey::new(Uuid::now_v7(), InnerKeyAlg::Rs512.into())
    }

    fn create_rsa_standlong_key() -> StandloneKey {
        RsaKey::new(Uuid::now_v7(), InnerKeyAlg::Rs512.into())
            .try_into()
            .unwrap()
    }

    // NOTE: AI-generated test
    fn now_fixed() -> DateTime<FixedOffset> {
        Utc::now().fixed_offset()
    }

    // NOTE: AI-generated test
    fn put_key_direct(keybox: &mut KeyBox, key: StandloneKey, option: KeyOption) {
        let StandloneKey {
            id,
            key_alg,
            secret,
        } = key;

        let KeyOption {
            created_at,
            activated_at,
            retired_at,
            expires_at,
        } = option;

        let now = Utc::now();
        let status = if expires_at.is_some_and(|t| now >= t) || retired_at.is_some_and(|t| now >= t)
        {
            KeyStatus::Retired
        } else if activated_at.is_none_or(|t| now >= t) {
            KeyStatus::Active
        } else {
            KeyStatus::Pending
        };

        let key = Key {
            id,
            key_alg: key_alg.into(),
            status,
            created_at,
            activated_at,
            retired_at,
            revoked_at: None,
            expires_at,
            secret,
            application_id: keybox.application_id,
        };

        keybox.keys.write().insert(id, key);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_without_activated_at_is_active() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standlong_key();
        let key_id = key.id;

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now_fixed(),
                activated_at: None,
                retired_at: None,
                expires_at: None,
            },
        );

        let stored_key = unsafe { keybox.get_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Active);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_with_past_activated_at_is_active() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standlong_key();
        let key_id = key.id;
        let past = now_fixed() - Duration::hours(1);

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now_fixed(),
                activated_at: Some(past),
                retired_at: None,
                expires_at: None,
            },
        );

        let stored_key = unsafe { keybox.get_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Active);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_with_future_activated_at_is_pending() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standlong_key();
        let key_id = key.id;
        let future = now_fixed() + Duration::hours(1);

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now_fixed(),
                activated_at: Some(future),
                retired_at: None,
                expires_at: None,
            },
        );

        let stored_key = unsafe { keybox.get_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Pending);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_with_past_expires_at_is_retired() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standlong_key();
        let key_id = key.id;
        let past = now_fixed() - Duration::hours(1);

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now_fixed(),
                activated_at: None,
                retired_at: None,
                expires_at: Some(past),
            },
        );

        let stored_key = unsafe { keybox.get_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Retired);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_with_past_retired_at_is_retired() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standlong_key();
        let key_id = key.id;
        let past = now_fixed() - Duration::hours(1);

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now_fixed(),
                activated_at: None,
                retired_at: Some(past),
                expires_at: None,
            },
        );

        let stored_key = unsafe { keybox.get_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Retired);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_expires_at_takes_precedence_over_activated_at() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standlong_key();
        let key_id = key.id;
        let past = now_fixed() - Duration::hours(1);
        let future = now_fixed() + Duration::hours(1);

        // `activated_at` is in the future (should be Pending), but expires_at has already passed
        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now_fixed(),
                activated_at: Some(future),
                retired_at: None,
                expires_at: Some(past),
            },
        );

        let stored_key = unsafe { keybox.get_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Retired);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_retired_at_takes_precedence_over_activated_at() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standlong_key();
        let key_id = key.id;
        let past = now_fixed() - Duration::hours(1);

        // `activated_at` is in the past (should be Active), but retired_at has already passed
        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now_fixed(),
                activated_at: Some(past - Duration::hours(1)),
                retired_at: Some(past),
                expires_at: None,
            },
        );

        let stored_key = unsafe { keybox.get_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Retired);
    }

    #[test]
    fn test_keybox_usage() {
        let mut keybox = KeyBox::new(Uuid::nil());

        // Put key
        keybox
            .put_key(
                create_rsa_key(),
                KeyOption {
                    ..Default::default()
                },
            )
            .unwrap();
    }
}
