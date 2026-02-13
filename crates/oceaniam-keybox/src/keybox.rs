use chrono::{DateTime, FixedOffset, Utc};
use im::HashMap;
use oceaniam_database::model::{key_boxes::Model as Key, sea_orm_active_enums::KeyStatus};
use serde_json::Value;
use uuid::Uuid;

use crate::{error::Error, key_alg::KeyAlg};

#[derive(Debug)]
pub struct StandaloneKey {
    pub key_id: Uuid,
    pub key_alg: KeyAlg,
    pub secret: Value,
}

impl From<Key> for StandaloneKey {
    fn from(
        Key {
            id,
            key_alg,
            secret,
            ..
        }: Key,
    ) -> Self {
        StandaloneKey {
            key_id: id,
            key_alg: key_alg.into(),
            secret,
        }
    }
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

impl StatusMaskedKey {
    pub fn into_key<T>(self) -> Option<T>
    where
        T: From<StandaloneKey>,
    {
        match self {
            StatusMaskedKey::Active(key)
            | StatusMaskedKey::Pending(key)
            | StatusMaskedKey::Retired(key) => Some(StandaloneKey::from(key).into()),

            StatusMaskedKey::Revoked => None,
        }
    }
}

/// [KeyBox] is used to manage multiple keys, providing expiration checking and key management functionality
#[derive(Debug, Clone)]
pub struct KeyBox {
    /// Belong to application
    application_id: Uuid,

    /// Stores all keys with [Key::id] as the key
    keys: HashMap<Uuid, Key>,
}

impl KeyBox {
    pub fn new(application_id: Uuid) -> Self {
        Self::with_keys(application_id, HashMap::default())
    }

    pub fn with_keys(application_id: Uuid, keys: HashMap<Uuid, Key>) -> Self {
        Self {
            application_id,
            keys,
        }
    }

    pub fn put_key<T>(&mut self, key: T) -> Result<(), Error>
    where
        T: TryInto<StandaloneKey, Error = Error>,
    {
        self.put_key_with_option(
            key,
            KeyOption {
                ..Default::default()
            },
        )
    }

    /// Adds a new key
    pub fn put_key_with_option<T>(
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
        T: TryInto<StandaloneKey, Error = Error>,
    {
        let StandaloneKey {
            key_id: id,
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

        self.keys.insert(id, key);
        Ok(())
    }

    /// Gets a key by key_id
    ///
    /// # Safety
    ///
    /// This function doesn't check the key is expired.
    pub unsafe fn get_key_unsafe(&self, key_id: &Uuid) -> Option<Key> {
        self.keys.get(key_id).cloned()
    }

    pub fn get_key(&self, key_id: &Uuid) -> Option<StatusMaskedKey> {
        unsafe { self.get_key_unsafe(key_id).map(StatusMaskedKey::from) }
    }

    /// Removes the specified key
    pub fn remove_key(&mut self, key_id: &Uuid) -> Option<Key> {
        self.keys.remove(key_id)
    }

    pub fn get_keys(&self) -> &HashMap<Uuid, Key> {
        &self.keys
    }
}

#[cfg(test)]
mod tests {
    use crate::key::rsa_key::RsaKey;

    use super::*;
    use chrono::Duration;
    use oceaniam_database::model::sea_orm_active_enums::KeyAlg as InnerKeyAlg;

    fn create_rsa_key() -> RsaKey {
        RsaKey::new(Uuid::now_v7(), InnerKeyAlg::Rs512)
    }

    fn create_rsa_standalone_key() -> StandaloneKey {
        RsaKey::new(Uuid::now_v7(), InnerKeyAlg::Rs512)
            .try_into()
            .unwrap()
    }

    // NOTE: AI-generated test
    fn now_fixed() -> DateTime<FixedOffset> {
        Utc::now().fixed_offset()
    }

    // NOTE: AI-generated test
    fn put_key_direct(keybox: &mut KeyBox, key: StandaloneKey, option: KeyOption) {
        let StandaloneKey {
            key_id: id,
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

        keybox.keys.insert(id, key);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_without_activated_at_is_active() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;

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
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
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
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
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
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
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
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
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
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
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
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
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
            .put_key_with_option(
                create_rsa_key(),
                KeyOption {
                    ..Default::default()
                },
            )
            .unwrap();
    }
}
