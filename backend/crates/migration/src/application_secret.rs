//! Adapts the current HMAC key-list configuration to the append-only v1 migration.
//!
//! `m20260726_142405_hash_application_secrets` predates the list syntax and reads a legacy
//! version-specific environment variable. Keep that committed migration unchanged without
//! exposing its historical variable name as supported user configuration.

use std::sync::LazyLock;

use oceaniam_application_secret::ApplicationSecretHmacKey;
use sea_orm_migration::prelude::*;
use zeroize::Zeroizing;

const HMAC_KEYS_ENV: &str = "OCEANIAM_APPLICATION_SECRET_HMAC__KEYS";
const LEGACY_HMAC_KEY_ENV_V1: &str = "OCEANIAM_APPLICATION_SECRET_HMAC__KEYS__1";
static LEGACY_ENV_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

struct RemoveLegacyKey;

impl Drop for RemoveLegacyKey {
    fn drop(&mut self) {
        // SAFETY: LEGACY_ENV_LOCK serializes all accesses made by this adapter.
        unsafe { std::env::remove_var(LEGACY_HMAC_KEY_ENV_V1) };
    }
}

fn version_one_key(entries: &str) -> Result<Zeroizing<String>, String> {
    let mut version_one = None;

    for entry in entries.split(',') {
        let (version, key) = entry
            .split_once(':')
            .ok_or_else(|| "every HMAC key entry must use `version:hex`".to_owned())?;
        if version.trim() != "1" {
            continue;
        }
        if version_one.is_some() {
            return Err("duplicate HMAC key version 1".to_owned());
        }
        version_one = Some(Zeroizing::new(key.trim().to_owned()));
    }

    let key = version_one.ok_or_else(|| "HMAC key version 1 is required".to_owned())?;
    ApplicationSecretHmacKey::from_hex_owned(key.as_str().to_owned())
        .map_err(|error| format!("invalid HMAC key version 1: {error}"))?;
    Ok(key)
}

/// Supplies v1 from the current list syntax to the historical migration without editing it.
pub struct ApplicationSecretHmacMigration<M>(M);

impl<M> ApplicationSecretHmacMigration<M> {
    pub fn new(inner: M) -> Self {
        Self(inner)
    }
}

impl<M> MigrationName for ApplicationSecretHmacMigration<M>
where
    M: MigrationTrait,
{
    fn name(&self) -> &str {
        self.0.name()
    }
}

#[async_trait::async_trait]
impl<M> MigrationTrait for ApplicationSecretHmacMigration<M>
where
    M: MigrationTrait,
{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _lock = LEGACY_ENV_LOCK.lock().await;
        let entries = Zeroizing::new(std::env::var(HMAC_KEYS_ENV).map_err(|_| {
            DbErr::Custom(format!(
                "environment variable {HMAC_KEYS_ENV} is required for this migration"
            ))
        })?);
        let key = version_one_key(&entries)
            .map_err(|error| DbErr::Custom(format!("invalid {HMAC_KEYS_ENV}: {error}")))?;

        // SAFETY: LEGACY_ENV_LOCK serializes the temporary value across the inner migration.
        unsafe { std::env::set_var(LEGACY_HMAC_KEY_ENV_V1, key.as_str()) };
        let _remove_legacy_key = RemoveLegacyKey;
        self.0.up(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.0.down(manager).await
    }
}

#[cfg(test)]
mod tests {
    use super::version_one_key;

    // NOTE: AI-generated test
    #[test]
    fn extracts_v1_from_the_current_key_list() {
        let first = "ab".repeat(32);
        let second = "cd".repeat(32);
        let key = version_one_key(&format!("2:{second}, 1 : {first}"))
            .expect("extract version one")
            .as_str()
            .to_owned();
        assert_eq!(key, first);
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_missing_duplicate_and_malformed_v1_entries() {
        let key = "ab".repeat(32);
        assert!(version_one_key(&format!("2:{key}")).is_err());
        assert!(version_one_key(&format!("1:{key},1:{key}")).is_err());
        assert!(version_one_key(&key).is_err());
        assert!(version_one_key("1:00").is_err());
    }
}
