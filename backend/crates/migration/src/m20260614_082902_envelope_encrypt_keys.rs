use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305, aead::Aead};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const KEK_ENV_VAR: &str = "OCEANIAM_MASTER_KEY";
const KEK_VERSION_CURRENT: u32 = 1;

struct EncryptedSecret {
    nonce: [u8; 24],
    ciphertext: Vec<u8>,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();
        let master_key_hex = std::env::var(KEK_ENV_VAR).map_err(|_| {
            DbErr::Custom(format!(
                "environment variable {KEK_ENV_VAR} is required for this migration"
            ))
        })?;
        let master_key = parse_master_key(&master_key_hex)?;

        let txn = connection.begin().await?;
        let rows = txn
            .query_all(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT id, secret FROM key_boxes",
                [],
            ))
            .await?;

        for row in rows {
            let id: uuid::Uuid = row.try_get("", "id")?;
            let secret: serde_json::Value = row.try_get("", "secret")?;

            if secret.get("ciphertext").is_some() {
                continue;
            }

            let pem = secret
                .get("pem")
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    DbErr::Custom(format!(
                        "key_boxes id={id} has neither ciphertext nor legacy pem field"
                    ))
                })?;

            let blob = encrypt_secret(&master_key, pem.as_bytes())
                .map_err(|e| DbErr::Custom(format!("encrypt key_boxes id={id}: {e}")))?;

            let encrypted_secret = serde_json::json!({
                "nonce": B64.encode(blob.nonce),
                "ciphertext": B64.encode(&blob.ciphertext),
                "key_version": KEK_VERSION_CURRENT,
            });

            txn.execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE key_boxes SET secret = $1 WHERE id = $2",
                vec![encrypted_secret.into(), id.into()],
            ))
            .await?;
        }

        txn.execute_unprepared(include_str!(
            "./m20260614_082902_envelope_encrypt_keys/up.sql"
        ))
        .await?;

        txn.commit().await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!(
                "./m20260614_082902_envelope_encrypt_keys/down.sql"
            ))
            .await?;

        Ok(())
    }
}

fn parse_master_key(hex_str: &str) -> Result<[u8; 32], DbErr> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| DbErr::Custom(format!("invalid {KEK_ENV_VAR} hex: {e}")))?;
    let key: [u8; 32] = bytes
        .try_into()
        .map_err(|_| DbErr::Custom(format!("{KEK_ENV_VAR} must decode to 32 bytes")))?;
    Ok(key)
}

fn encrypt_secret(master_key: &[u8; 32], plaintext: &[u8]) -> Result<EncryptedSecret, String> {
    let cipher = XChaCha20Poly1305::new_from_slice(master_key).map_err(|e| e.to_string())?;
    let nonce = XChaCha20Poly1305::generate_nonce(&mut rand::thread_rng());
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| e.to_string())?;

    Ok(EncryptedSecret {
        nonce: nonce.into(),
        ciphertext,
    })
}
