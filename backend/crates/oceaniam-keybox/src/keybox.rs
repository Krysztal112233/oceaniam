use crate::error::Error;
use chrono::{DateTime, FixedOffset, Utc};
use im::HashMap;
use itertools::Itertools;
use oceaniam_database::{
    helper::{SafeTransactionConnectionTrait, key_boxes::KeyBoxesHelper},
    model::{
        key_boxes::Model as Key,
        prelude::KeyBoxes,
        sea_orm_active_enums::{self, KeyStatus},
    },
};
use sea_orm::IntoActiveModel;
use serde_json::Value;
use tracing::error;
use uuid::Uuid;

use crate::{
    key::{TryIntoJwk, TryIntoKeyModel, rsa_key::RsaKey},
    key_alg::KeyAlg,
};

/// A standalone key representation that contains the essential key information
/// without the full metadata from the database model
#[derive(Debug, Clone)]
pub struct StandaloneKey {
    /// Unique identifier for the key
    pub key_id: Uuid,
    /// The algorithm used by this key (e.g., RS256, RS512, etc.)
    pub key_alg: KeyAlg,
    /// The secret key material stored as JSON value
    pub secret: Value,
}

impl From<Key> for StandaloneKey {
    /// Converts a database Key model into a StandaloneKey
    ///
    /// Extracts the essential key information while discarding metadata
    /// like status timestamps and application_id
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

/// Configuration options for creating a new key
#[derive(Debug)]
pub struct KeyOption {
    /// Timestamp when the key was created
    pub created_at: DateTime<FixedOffset>,

    /// Timestamp when the key becomes active
    pub activated_at: DateTime<FixedOffset>,

    /// Timestamp when the key is retired
    pub retired_at: DateTime<FixedOffset>,

    /// Timestamp when the key expires
    pub expires_at: DateTime<FixedOffset>,
}

impl Default for KeyOption {
    /// Creates default key options with current time as creation timestamp
    fn default() -> Self {
        let now: DateTime<FixedOffset> = Utc::now().into();
        Self {
            created_at: now,
            activated_at: now,
            retired_at: now + chrono::Duration::days(30),
            expires_at: now + chrono::Duration::days(60),
        }
    }
}

/// [KeyBox] is used to manage multiple keys, providing expiration checking and key management functionality
#[derive(Debug, Clone)]
pub struct KeyBox {
    /// Belong to tenant
    tenant_id: Uuid,

    /// Stores all keys with [Key::id] as the key
    keys: HashMap<Uuid, Key>,
}

impl KeyBox {
    /// Creates a new empty KeyBox for the specified tenant
    pub fn new(tenant_id: Uuid) -> Self {
        Self::with_keys(tenant_id, HashMap::default())
    }

    /// Creates a KeyBox with the specified keys
    pub fn with_keys(tenant_id: Uuid, keys: HashMap<Uuid, Key>) -> Self {
        Self { tenant_id, keys }
    }

    /// Adds a new key with default options.
    ///
    /// Returns an error if a key with the same ID already exists in this keybox.
    pub fn add_key<T>(&mut self, key: T) -> Result<(), Error>
    where
        T: TryIntoKeyModel,
    {
        self.add_key_with_option(key, KeyOption::default())
    }

    /// Adds a new key with custom options.
    ///
    /// Returns an error if a key with the same ID already exists in this keybox.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to add, must implement `TryIntoKeyModel`
    /// * `options` - Configuration options for the key lifecycle
    pub fn add_key_with_option<T>(&mut self, key: T, options: KeyOption) -> Result<(), Error>
    where
        T: TryIntoKeyModel,
    {
        let key = key.try_into_key_model(self.tenant_id, options)?;

        if self.keys.contains_key(&key.id) {
            return Err(Error::KeyAlreadyExists {
                id: key.id.to_string(),
                location: snafu::location!(),
            });
        }

        self.keys.insert(key.id, key);
        Ok(())
    }

    /// Gets a key by key_id
    ///
    /// # Safety
    ///
    /// This function doesn't check the key is expired.
    pub unsafe fn get_raw_key_unsafe(&self, key_id: &Uuid) -> Option<Key> {
        self.keys.get(key_id).cloned()
    }

    /// Gets a key by key_id and converts it to StandaloneKey
    ///
    /// # Safety
    ///
    /// This function doesn't check if the key is expired.
    pub fn get_raw_key(&self, key_id: &Uuid) -> Option<StandaloneKey> {
        unsafe { self.get_raw_key_unsafe(key_id).map(Into::into) }
    }

    /// Gets a key by key_id and converts it to the specified type
    ///
    /// # Arguments
    ///
    /// * `key_id` - The UUID of the key to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(Ok(key))` if the key exists and can be converted,
    /// `Some(Err(e))` if conversion fails, or `None` if key doesn't exist
    pub fn get_key<T>(&self, key_id: &Uuid) -> Option<Result<T, Error>>
    where
        T: TryFrom<StandaloneKey, Error = Error>,
    {
        self.get_raw_key(key_id).map(T::try_from)
    }

    /// Revokes the specified key by setting its status to `Revoked`
    /// and recording the revocation timestamp.
    ///
    /// Returns an error if the key does not exist in this keybox.
    pub fn revoke_key(&mut self, key_id: &Uuid) -> Result<(), Error> {
        let Some(mut key) = self.keys.get(key_id).cloned() else {
            return Err(Error::KeyNotFound {
                id: *key_id,
                location: snafu::location!(),
            });
        };

        key.status = KeyStatus::Revoked;
        key.revoked_at = Some(Utc::now().into());

        self.keys.insert(*key_id, key);
        Ok(())
    }

    /// Generates a new key and adds it to this keybox.
    ///
    /// This is a pure in-memory operation — the caller is responsible for
    /// persisting via [`KeyBox::write_to`].
    pub fn rotate_key(&mut self) -> Result<Key, Error> {
        let rsa_key = RsaKey::new(Uuid::now_v7(), sea_orm_active_enums::KeyAlg::Ps512);
        let key_id = rsa_key.key_id();

        self.add_key_with_option(rsa_key, KeyOption::default())?;

        // SAFETY: the key was just inserted, it exists and is not expired
        unsafe { self.get_raw_key_unsafe(&key_id) }.ok_or_else(|| Error::Internal {
            msg: "key was inserted but cannot be retrieved".into(),
            location: snafu::location!(),
        })
    }

    /// Returns all keys in the keybox
    pub fn get_keys(&self) -> &HashMap<Uuid, Key> {
        &self.keys
    }

    /// Refreshes each key's status based on its timestamps and current time.
    ///
    /// A previously [`Active`](KeyStatus::Active) key becomes [`Retired`](KeyStatus::Retired) once
    /// its `retired_at` or `expires_at` has passed.  [`Revoked`](KeyStatus::Revoked) keys are never
    /// changed.
    ///
    /// Returns `true` if any status was updated.
    pub fn refresh_statuses(&mut self) -> bool {
        let now: DateTime<FixedOffset> = Utc::now().into();
        let mut changed = false;

        let updates: Vec<(Uuid, KeyStatus)> = self
            .keys
            .iter()
            .filter_map(|(id, key)| {
                if key.revoked_at.is_some() {
                    return None;
                }

                let new_status = if now >= key.expires_at || now >= key.retired_at {
                    KeyStatus::Retired
                } else if now >= key.activated_at {
                    KeyStatus::Active
                } else {
                    KeyStatus::Pending
                };

                (new_status != key.status).then_some((*id, new_status))
            })
            .collect();

        for (id, status) in &updates {
            if let Some(mut key) = self.keys.get(id).cloned() {
                key.status = status.clone();
                self.keys.insert(*id, key);
                changed = true;
            }
        }

        changed
    }

    /// Gets the latest key with the specified status
    ///
    /// Returns the key with the most recent `activated_at` timestamp
    /// that matches the given status
    pub fn get_latest_raw_key(&self, status: KeyStatus) -> Option<StandaloneKey> {
        self.keys
            .values()
            .filter(|it| it.status == status)
            .sorted_by(|a, b| Ord::cmp(&b.activated_at, &a.activated_at))
            .cloned()
            .map(StandaloneKey::from)
            .next()
    }

    /// Gets the latest key with the specified status and converts it to the target type
    pub fn get_latest_key<T>(&self, status: KeyStatus) -> Option<Result<T, Error>>
    where
        T: TryFrom<StandaloneKey, Error = Error>,
    {
        self.get_latest_raw_key(status).map(T::try_from)
    }

    /// Writes all keys in this keybox to the database
    ///
    /// Persists all key changes to the database for the application
    pub async fn write_to(
        &self,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let vec = self
            .keys
            .iter()
            .map(|(_, model)| model)
            .cloned()
            .map(|it| it.into_active_model());

        KeyBoxes::update_application_keys(self.tenant_id, vec, database).await?;

        Ok(())
    }

    /// Returns the tenant ID this keybox belongs to
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }
}

impl From<KeyBox> for oceaniam_auth::jwks::JwkSet {
    /// Converts a KeyBox into a JWK Set (JSON Web Key Set)
    ///
    /// Only includes non-revoked RSA keys. Other key types are ignored.
    /// Failed conversions are logged but don't stop the process.
    fn from(value: KeyBox) -> Self {
        let keys = value
            .keys
            .values()
            .filter(|it| it.status != KeyStatus::Revoked)
            .cloned()
            .flat_map(|it| match it.key_alg {
                sea_orm_active_enums::KeyAlg::Rs256
                | sea_orm_active_enums::KeyAlg::Rs384
                | sea_orm_active_enums::KeyAlg::Rs512
                | sea_orm_active_enums::KeyAlg::Ps256
                | sea_orm_active_enums::KeyAlg::Ps384
                | sea_orm_active_enums::KeyAlg::Ps512 => RsaKey::try_from(it)
                    .inspect_err(|e| error!("{e}"))
                    .map(|it| it.try_into_jwk()),
            })
            .flatten()
            .collect();

        Self { keys }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    use crate::key::rsa_key::RsaKey;

    use chrono::{Duration, Utc};
    use jsonwebtoken::{Algorithm, Header, TokenData, Validation};
    use oceaniam_auth::{
        jwks::JwkSet,
        jwt::{ClaimHelper, JwtCodec, SystemClaim},
    };
    use oceaniam_database::model::sea_orm_active_enums::KeyAlg as InnerKeyAlg;
    use tap::Tap;

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

        let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let status = if now >= expires_at || now >= retired_at {
            KeyStatus::Retired
        } else if now >= activated_at {
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
            tenant_id: keybox.tenant_id,
        };

        keybox.keys.insert(id, key);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_without_activated_at_is_active() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;

        let now = now_fixed();

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now,
                activated_at: now,
                retired_at: now + Duration::hours(2),
                expires_at: now + Duration::hours(3),
            },
        );

        let stored_key = unsafe { keybox.get_raw_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Active);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_with_past_activated_at_is_active() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
        let now = now_fixed();
        let past = now - Duration::hours(1);

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now,
                activated_at: past,
                retired_at: now + Duration::hours(2),
                expires_at: now + Duration::hours(3),
            },
        );

        let stored_key = unsafe { keybox.get_raw_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Active);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_with_future_activated_at_is_pending() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
        let now = now_fixed();
        let future = now + Duration::hours(1);

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now,
                activated_at: future,
                retired_at: future + Duration::hours(1),
                expires_at: future + Duration::hours(2),
            },
        );

        let stored_key = unsafe { keybox.get_raw_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Pending);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_with_past_expires_at_is_retired() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
        let now = now_fixed();
        let past = now - Duration::hours(1);

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now,
                activated_at: past - Duration::hours(1),
                retired_at: past - Duration::hours(1),
                expires_at: past,
            },
        );

        let stored_key = unsafe { keybox.get_raw_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Retired);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_put_key_with_past_retired_at_is_retired() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
        let now = now_fixed();
        let past = now - Duration::hours(1);

        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now,
                activated_at: past - Duration::hours(1),
                retired_at: past,
                expires_at: now + Duration::hours(1),
            },
        );

        let stored_key = unsafe { keybox.get_raw_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Retired);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_expires_at_takes_precedence_over_activated_at() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
        let now = now_fixed();
        let past = now - Duration::hours(1);
        let future = now + Duration::hours(1);

        // `activated_at` is in the future (should be Pending), but expires_at has already passed
        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now,
                activated_at: future,
                retired_at: future + Duration::hours(1),
                expires_at: past,
            },
        );

        let stored_key = unsafe { keybox.get_raw_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Retired);
    }

    // NOTE: AI-generated test
    #[test]
    fn test_retired_at_takes_precedence_over_activated_at() {
        let mut keybox = KeyBox::new(Uuid::now_v7());
        let key = create_rsa_standalone_key();
        let key_id = key.key_id;
        let now = now_fixed();
        let past = now - Duration::hours(1);

        // `activated_at` is in the past (should be Active), but retired_at has already passed
        put_key_direct(
            &mut keybox,
            key,
            KeyOption {
                created_at: now,
                activated_at: past - Duration::hours(1),
                retired_at: past,
                expires_at: now + Duration::hours(1),
            },
        );

        let stored_key = unsafe { keybox.get_raw_key_unsafe(&key_id) }.unwrap();
        assert_eq!(stored_key.status, KeyStatus::Retired);
    }

    #[test]
    fn test_keybox_usage() {
        let mut keybox = KeyBox::new(Uuid::nil());

        let key = create_rsa_key();

        // Put key
        keybox.add_key(key.clone()).unwrap();

        let Some(key) = keybox.get_raw_key(&key.key_id()) else {
            panic!("EXPECT ACTIVE KEY BUT CANNOT GET IT")
        };

        let rsa_key = RsaKey::try_from(key).unwrap();

        let jwt = rsa_key
            .encode(
                Header::new(Algorithm::RS512),
                SystemClaim::new(Uuid::now_v7(), 60, None, None),
            )
            .unwrap();

        let _claim: TokenData<SystemClaim> = rsa_key
            .decode(
                jwt.as_bytes(),
                &Validation::default().tap_mut(|it| {
                    it.algorithms = vec![Algorithm::RS512, Algorithm::RS256, Algorithm::RS384]
                }),
            )
            .unwrap();
    }

    #[test]
    fn test_keybox_into_jwks() {
        let mut keybox = KeyBox::new(Uuid::nil());

        // Put key
        keybox.add_key(create_rsa_key()).unwrap();
        keybox.add_key(create_rsa_key()).unwrap();
        keybox.add_key(create_rsa_key()).unwrap();

        let _ = JwkSet::from(keybox);
    }
}
