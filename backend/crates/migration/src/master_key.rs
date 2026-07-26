//! Master-key validation for migrations that encrypt persisted secrets.
//!
//! This module keeps committed migration files unchanged while ensuring the
//! configured key is a non-zero, 32-byte hexadecimal value before `up` runs.

use sea_orm_migration::prelude::*;
use zeroize::Zeroizing;

const MASTER_KEY_ENV_VAR: &str = "OCEANIAM_MASTER_KEY";

/// Validates an owned hexadecimal master key and zeroizes its source buffer.
pub fn validate_master_key(hex_string: String) -> Result<(), String> {
    let hex_string = Zeroizing::new(hex_string);

    if hex_string.is_empty() {
        return Err("must be configured and must not be empty".to_owned());
    }

    if hex_string.len() != 64 {
        return Err("must contain exactly 64 hexadecimal characters".to_owned());
    }

    let mut key = Zeroizing::new([0u8; 32]);
    hex::decode_to_slice(hex_string.as_str(), key.as_mut()).map_err(|error| error.to_string())?;

    if key.iter().all(|byte| *byte == 0) {
        return Err("must not be all zeroes".to_owned());
    }

    Ok(())
}

/// Wraps a migration with master-key validation while preserving its name and rollback behavior.
pub struct MasterKeyValidatedMigration<M>(M);

impl<M> MasterKeyValidatedMigration<M> {
    /// Wraps the migration whose `up` operation requires a valid master key.
    pub fn new(inner: M) -> Self {
        Self(inner)
    }
}

impl<M> MigrationName for MasterKeyValidatedMigration<M>
where
    M: MigrationTrait,
{
    fn name(&self) -> &str {
        self.0.name()
    }
}

#[async_trait::async_trait]
impl<M> MigrationTrait for MasterKeyValidatedMigration<M>
where
    M: MigrationTrait,
{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let master_key = std::env::var(MASTER_KEY_ENV_VAR).map_err(|_| {
            DbErr::Custom(format!(
                "environment variable {MASTER_KEY_ENV_VAR} is required for this migration"
            ))
        })?;
        validate_master_key(master_key)
            .map_err(|error| DbErr::Custom(format!("invalid {MASTER_KEY_ENV_VAR}: {error}")))?;

        self.0.up(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        self.0.down(manager).await
    }
}

#[cfg(test)]
mod tests {
    use super::validate_master_key;

    // NOTE: AI-generated test
    #[test]
    fn rejects_empty_master_key_with_clear_error() {
        assert_eq!(
            validate_master_key(String::new()).unwrap_err(),
            "must be configured and must not be empty"
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_all_zero_master_key() {
        assert!(
            validate_master_key(
                "0000000000000000000000000000000000000000000000000000000000000000".to_owned()
            )
            .is_err()
        );
    }
}
