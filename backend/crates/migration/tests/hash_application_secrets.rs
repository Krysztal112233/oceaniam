use migration::{Migrator, MigratorTrait};
use oceaniam_application_secret::{ApplicationSecretHmacKey, ApplicationSecretKeyring};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use uuid::Uuid;

const TEST_MASTER_KEY_HEX: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const TEST_HMAC_KEY_HEX: &str = "89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567";
const HMAC_KEY_ENV: &str = "OCEANIAM_APPLICATION_SECRET_HMAC__KEYS__1";
const LEGACY_SECRET: &str = "app_01234567890123456789012345678901";
const SECOND_LEGACY_SECRET: &str = "app_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const MIGRATION_VERSION: &str = "m20260726_142405_hash_application_secrets";

struct TestSchema {
    base_dsn: String,
    name: String,
}

impl Drop for TestSchema {
    fn drop(&mut self) {
        let base_dsn = self.base_dsn.clone();
        let name = self.name.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().expect("create cleanup runtime");
            runtime.block_on(async move {
                let base = Database::connect(&base_dsn)
                    .await
                    .expect("connect for schema cleanup");
                base.execute(Statement::from_string(
                    DatabaseBackend::Postgres,
                    format!("DROP SCHEMA IF EXISTS {name} CASCADE"),
                ))
                .await
                .expect("drop migration test schema");
                base.close().await.expect("close cleanup connection");
            });
        })
        .join()
        .expect("join schema cleanup thread");
    }
}

async fn isolated_database() -> (TestSchema, DatabaseConnection) {
    let base_dsn = std::env::var("OCEANIAM_TEST_DATABASE_DSN")
        .or_else(|_| std::env::var("OCEANIAM_DATABASE__DSN"))
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/postgres".to_owned());
    let name = format!("test_app_secret_migration_{}", Uuid::now_v7().simple());
    let base = Database::connect(&base_dsn).await.unwrap_or_else(|error| {
        panic!(
            "failed to connect to the migration test database; set \
             OCEANIAM_TEST_DATABASE_DSN, OCEANIAM_DATABASE__DSN, or DATABASE_URL: {error}"
        )
    });
    base.execute(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("CREATE SCHEMA {name}"),
    ))
    .await
    .unwrap();
    base.close().await.unwrap();

    let database = Database::connect(
        ConnectOptions::new(&base_dsn)
            .set_schema_search_path(format!("{name},public"))
            .to_owned(),
    )
    .await
    .unwrap();

    (TestSchema { base_dsn, name }, database)
}

async fn assert_legacy_schema_intact(database: &DatabaseConnection, expected_secrets: &[&str]) {
    let columns: Vec<String> = database
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT column_name FROM information_schema.columns \
             WHERE table_schema = current_schema() AND table_name = 'application_secrets'"
                .to_owned(),
        ))
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.try_get("", "column_name").unwrap())
        .collect();
    assert!(columns.iter().any(|column| column == "secret"));
    assert!(!columns.iter().any(|column| column == "secret_verifier"));

    let stored: Vec<String> = database
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT secret FROM application_secrets ORDER BY secret".to_owned(),
        ))
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.try_get("", "secret").unwrap())
        .collect();
    let mut expected: Vec<String> = expected_secrets
        .iter()
        .map(|secret| (*secret).to_owned())
        .collect();
    expected.sort();
    assert_eq!(stored, expected);
}

// NOTE: AI-generated test
#[tokio::test]
async fn migration_validates_inputs_rolls_back_and_is_idempotent() {
    dotenvy::dotenv().ok();

    // SAFETY: this integration-test binary has one test, so its sequential environment changes
    // cannot race another test in this process. All values are deterministic test-only keys.
    unsafe {
        std::env::set_var("OCEANIAM_MASTER_KEY", TEST_MASTER_KEY_HEX);
        std::env::set_var("MIGRATION_DEFAULT_ROOT_PASSWORD", "migration-test-password");
        std::env::remove_var(HMAC_KEY_ENV);
    }

    let (_schema, database) = isolated_database().await;
    let migration_count = Migrator::migrations().len();
    Migrator::up(&database, Some((migration_count - 1) as u32))
        .await
        .unwrap();

    let tenant_id = Uuid::now_v7();
    let application_id = Uuid::now_v7();
    let secret_id = Uuid::now_v7();
    let second_secret_id = Uuid::now_v7();
    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO tenants (id, comment, created_at) VALUES ($1, NULL, now())",
            [tenant_id.into()],
        ))
        .await
        .unwrap();
    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO applications (id, comment, tenant_id, created_at) \
             VALUES ($1, NULL, $2, now())",
            vec![application_id.into(), tenant_id.into()],
        ))
        .await
        .unwrap();
    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO application_secrets (id, secret, created_at, revoked_at) \
             VALUES ($1, $2, now(), NULL)",
            vec![secret_id.into(), LEGACY_SECRET.into()],
        ))
        .await
        .unwrap();
    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO application_secret_bindings (secret_id, application_id) VALUES ($1, $2)",
            vec![secret_id.into(), application_id.into()],
        ))
        .await
        .unwrap();

    let missing_key = Migrator::up(&database, Some(1)).await.unwrap_err();
    assert!(missing_key.to_string().contains(HMAC_KEY_ENV));
    assert_legacy_schema_intact(&database, &[LEGACY_SECRET]).await;

    // SAFETY: sequential within this single test.
    unsafe { std::env::set_var(HMAC_KEY_ENV, "00") };
    let invalid_key = Migrator::up(&database, Some(1)).await.unwrap_err();
    assert!(invalid_key.to_string().contains("invalid"));
    assert_legacy_schema_intact(&database, &[LEGACY_SECRET]).await;

    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO application_secrets (id, secret, created_at, revoked_at) \
             VALUES ($1, $2, now(), NULL)",
            vec![second_secret_id.into(), "malformed".into()],
        ))
        .await
        .unwrap();
    // SAFETY: sequential within this single test.
    unsafe { std::env::set_var(HMAC_KEY_ENV, TEST_HMAC_KEY_HEX) };
    let malformed = Migrator::up(&database, Some(1)).await.unwrap_err();
    assert!(malformed.to_string().contains("invalid plaintext format"));
    assert_legacy_schema_intact(&database, &[LEGACY_SECRET, "malformed"]).await;

    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE application_secrets SET secret = $1 WHERE id = $2",
            vec![LEGACY_SECRET.into(), second_secret_id.into()],
        ))
        .await
        .unwrap();
    let duplicate = Migrator::up(&database, Some(1)).await.unwrap_err();
    assert!(
        duplicate
            .to_string()
            .contains("duplicate application secret")
    );
    assert_legacy_schema_intact(&database, &[LEGACY_SECRET, LEGACY_SECRET]).await;

    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE application_secrets SET secret = $1 WHERE id = $2",
            vec![SECOND_LEGACY_SECRET.into(), second_secret_id.into()],
        ))
        .await
        .unwrap();
    Migrator::up(&database, Some(1)).await.unwrap();

    let row = database
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT secret_prefix, secret_verifier, hmac_key_version \
             FROM application_secrets WHERE id = $1",
            [secret_id.into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let prefix: String = row.try_get("", "secret_prefix").unwrap();
    let verifier: Vec<u8> = row.try_get("", "secret_verifier").unwrap();
    let version: i32 = row.try_get("", "hmac_key_version").unwrap();
    let keyring = ApplicationSecretKeyring::new(
        1,
        [(
            1,
            ApplicationSecretHmacKey::from_hex_owned(TEST_HMAC_KEY_HEX.to_owned()).unwrap(),
        )],
    )
    .unwrap();

    assert_eq!(prefix, &LEGACY_SECRET[..12]);
    assert_eq!(verifier.len(), 32);
    assert_eq!(version, 1);
    assert!(keyring.verify(version, LEGACY_SECRET, &verifier).unwrap());

    let binding_count: i64 = database
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT count(*) AS count FROM application_secret_bindings WHERE secret_id = $1",
            [secret_id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "count")
        .unwrap();
    assert_eq!(binding_count, 1);

    let constraint_names: Vec<String> = database
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT conname FROM pg_constraint \
             WHERE conrelid = 'application_secrets'::regclass \
             AND conname IN ('ck_application_secrets_secret_verifier_length', \
                             'ck_application_secrets_secret_prefix_format') \
             ORDER BY conname"
                .to_owned(),
        ))
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.try_get("", "conname").unwrap())
        .collect();
    assert_eq!(
        constraint_names,
        vec![
            "ck_application_secrets_secret_prefix_format".to_owned(),
            "ck_application_secrets_secret_verifier_length".to_owned(),
        ]
    );

    let indexes: Vec<(String, String)> = database
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = current_schema() \
             AND tablename = 'application_secrets' \
             AND indexname IN ('idx_application_secrets_secret_prefix', \
                               'uq_application_secrets_hmac_verifier') \
             ORDER BY indexname"
                .to_owned(),
        ))
        .await
        .unwrap()
        .into_iter()
        .map(|row| {
            (
                row.try_get("", "indexname").unwrap(),
                row.try_get("", "indexdef").unwrap(),
            )
        })
        .collect();
    assert_eq!(indexes.len(), 2);
    assert_eq!(indexes[0].0, "idx_application_secrets_secret_prefix");
    assert_eq!(indexes[1].0, "uq_application_secrets_hmac_verifier");
    assert!(indexes[1].1.starts_with("CREATE UNIQUE INDEX"));

    let nullability: Vec<(String, String)> = database
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT column_name, is_nullable FROM information_schema.columns \
             WHERE table_schema = current_schema() AND table_name = 'application_secrets' \
             AND column_name IN ('secret_prefix', 'secret_verifier', 'hmac_key_version') \
             ORDER BY column_name"
                .to_owned(),
        ))
        .await
        .unwrap()
        .into_iter()
        .map(|row| {
            (
                row.try_get("", "column_name").unwrap(),
                row.try_get("", "is_nullable").unwrap(),
            )
        })
        .collect();
    assert_eq!(nullability.len(), 3);
    assert!(nullability.iter().all(|(_, nullable)| nullable == "NO"));

    let malformed_prefix_insert = database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO application_secrets \
             (id, created_at, revoked_at, secret_prefix, secret_verifier, hmac_key_version) \
             VALUES ($1, now(), NULL, $2, $3, 1)",
            vec![
                Uuid::now_v7().into(),
                "app_1234567!".into(),
                vec![7u8; 32].into(),
            ],
        ))
        .await;
    assert!(malformed_prefix_insert.is_err());

    let invalid_verifier_insert = database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO application_secrets \
             (id, created_at, revoked_at, secret_prefix, secret_verifier, hmac_key_version) \
             VALUES ($1, now(), NULL, $2, $3, 1)",
            vec![
                Uuid::now_v7().into(),
                "app_AbCd1234".into(),
                vec![7u8; 31].into(),
            ],
        ))
        .await;
    assert!(invalid_verifier_insert.is_err());

    let plaintext_column_count: i64 = database
        .query_one(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT count(*) AS count FROM information_schema.columns \
             WHERE table_schema = current_schema() \
             AND table_name = 'application_secrets' AND column_name = 'secret'"
                .to_owned(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "count")
        .unwrap();
    assert_eq!(plaintext_column_count, 0);

    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "DELETE FROM seaql_migrations WHERE version = $1",
            [MIGRATION_VERSION.into()],
        ))
        .await
        .unwrap();
    Migrator::up(&database, Some(1)).await.unwrap();

    database.close().await.unwrap();
}
