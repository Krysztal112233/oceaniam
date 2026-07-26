use std::collections::HashSet;

use oceaniam_application_secret::{
    ApplicationSecret, ApplicationSecretHmacKey, ApplicationSecretKeyring,
};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait};
use sea_orm_migration::prelude::*;

const HMAC_KEY_ENV_VAR_V1: &str = "OCEANIAM_APPLICATION_SECRET_HMAC__KEYS__1";
const HMAC_KEY_VERSION_V1: i32 = 1;
const IDX_APPLICATION_SECRETS_SECRET: &str = "idx_application_secrets_secret";
const IDX_APPLICATION_SECRETS_PREFIX: &str = "idx_application_secrets_secret_prefix";
const UQ_APPLICATION_SECRETS_VERIFIER: &str = "uq_application_secrets_hmac_verifier";
const CK_APPLICATION_SECRETS_VERIFIER_LENGTH: &str =
    "ck_application_secrets_secret_verifier_length";
const CK_APPLICATION_SECRETS_PREFIX_FORMAT: &str = "ck_application_secrets_secret_prefix_format";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let key_hex = std::env::var(HMAC_KEY_ENV_VAR_V1).map_err(|_| {
            DbErr::Custom(format!(
                "environment variable {HMAC_KEY_ENV_VAR_V1} is required for this migration"
            ))
        })?;
        let key = ApplicationSecretHmacKey::from_hex_owned(key_hex)
            .map_err(|error| DbErr::Custom(format!("invalid {HMAC_KEY_ENV_VAR_V1}: {error}")))?;
        let keyring = ApplicationSecretKeyring::new(HMAC_KEY_VERSION_V1, [(1, key)])
            .map_err(|error| DbErr::Custom(error.to_string()))?;

        let has_plaintext = manager.has_column("application_secrets", "secret").await?;
        let connection = manager.get_connection();
        let transaction = connection.begin().await?;

        transaction
            .execute_unprepared(
                "ALTER TABLE application_secrets
                 ADD COLUMN IF NOT EXISTS secret_prefix VARCHAR(12),
                 ADD COLUMN IF NOT EXISTS secret_verifier BYTEA,
                 ADD COLUMN IF NOT EXISTS hmac_key_version INTEGER",
            )
            .await?;

        if has_plaintext {
            let rows = transaction
                .query_all(Statement::from_string(
                    DatabaseBackend::Postgres,
                    "SELECT id, secret FROM application_secrets ORDER BY id".to_owned(),
                ))
                .await?;
            let mut verifiers = HashSet::with_capacity(rows.len());

            for row in rows {
                let id: uuid::Uuid = row.try_get("", "id")?;
                let plaintext: String = row.try_get("", "secret")?;
                let plaintext = ApplicationSecret::parse_owned(plaintext).map_err(|error| {
                    DbErr::Custom(format!(
                        "application_secrets id={id} has invalid plaintext format: {error}"
                    ))
                })?;
                let versioned = keyring
                    .verifier_for_current(plaintext.expose())
                    .map_err(|error| DbErr::Custom(error.to_string()))?;

                if !verifiers.insert(versioned.verifier) {
                    return Err(DbErr::Custom(format!(
                        "duplicate application secret or HMAC verifier detected at id={id}"
                    )));
                }

                transaction
                    .execute(Statement::from_sql_and_values(
                        DatabaseBackend::Postgres,
                        "UPDATE application_secrets
                         SET secret_prefix = $1, secret_verifier = $2, hmac_key_version = $3
                         WHERE id = $4",
                        vec![
                            plaintext.stored_prefix().to_owned().into(),
                            versioned.verifier.to_vec().into(),
                            versioned.hmac_key_version.into(),
                            id.into(),
                        ],
                    ))
                    .await?;
            }
        }

        transaction
            .execute_unprepared(
                "ALTER TABLE application_secrets
                 ALTER COLUMN secret_prefix SET NOT NULL,
                 ALTER COLUMN secret_verifier SET NOT NULL,
                 ALTER COLUMN hmac_key_version SET NOT NULL",
            )
            .await?;

        transaction
            .execute_unprepared(&format!(
                "DO $$ BEGIN
                    IF NOT EXISTS (
                        SELECT 1 FROM pg_constraint
                        WHERE conname = '{CK_APPLICATION_SECRETS_VERIFIER_LENGTH}'
                        AND conrelid = 'application_secrets'::regclass
                    ) THEN
                        ALTER TABLE application_secrets
                        ADD CONSTRAINT {CK_APPLICATION_SECRETS_VERIFIER_LENGTH}
                        CHECK (octet_length(secret_verifier) = 32);
                    END IF;
                    IF NOT EXISTS (
                        SELECT 1 FROM pg_constraint
                        WHERE conname = '{CK_APPLICATION_SECRETS_PREFIX_FORMAT}'
                        AND conrelid = 'application_secrets'::regclass
                    ) THEN
                        ALTER TABLE application_secrets
                        ADD CONSTRAINT {CK_APPLICATION_SECRETS_PREFIX_FORMAT}
                        CHECK (secret_prefix ~ '^app_[A-Za-z0-9]{{8}}$');
                    END IF;
                 END $$"
            ))
            .await?;

        let prefix_index = Index::create()
            .if_not_exists()
            .name(IDX_APPLICATION_SECRETS_PREFIX)
            .table(ApplicationSecrets::Table)
            .col(ApplicationSecrets::SecretPrefix)
            .to_owned();
        transaction
            .execute(transaction.get_database_backend().build(&prefix_index))
            .await?;

        let verifier_index = Index::create()
            .if_not_exists()
            .unique()
            .name(UQ_APPLICATION_SECRETS_VERIFIER)
            .table(ApplicationSecrets::Table)
            .col(ApplicationSecrets::HmacKeyVersion)
            .col(ApplicationSecrets::SecretVerifier)
            .to_owned();
        transaction
            .execute(transaction.get_database_backend().build(&verifier_index))
            .await?;

        let plaintext_index = Index::drop()
            .if_exists()
            .name(IDX_APPLICATION_SECRETS_SECRET)
            .table(ApplicationSecrets::Table)
            .to_owned();
        transaction
            .execute(transaction.get_database_backend().build(&plaintext_index))
            .await?;

        transaction
            .execute_unprepared("ALTER TABLE application_secrets DROP COLUMN IF EXISTS secret")
            .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "hash_application_secrets is irreversible; restore the pre-migration database backup"
                .to_owned(),
        ))
    }
}

#[derive(DeriveIden)]
enum ApplicationSecrets {
    Table,
    SecretPrefix,
    SecretVerifier,
    HmacKeyVersion,
}
