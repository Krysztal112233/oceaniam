use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use uuid::Uuid;

const TEST_MASTER_KEY_HEX: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const TEST_HMAC_KEY_HEX: &str = "89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567";
const MIGRATION_VERSION: &str = "m20261104_111941_extract_key_box_kek_version";

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
                base.execute_raw(Statement::from_string(
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
    let name = format!("test_kek_version_migration_{}", Uuid::now_v7().simple());
    let base = Database::connect(&base_dsn)
        .await
        .expect("connect to migration test database");
    base.execute_raw(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("CREATE SCHEMA {name}"),
    ))
    .await
    .expect("create isolated migration schema");
    base.close().await.expect("close base connection");

    let database = Database::connect(
        ConnectOptions::new(&base_dsn)
            .set_schema_search_path(format!("{name},public"))
            .to_owned(),
    )
    .await
    .expect("connect to isolated migration schema");

    (TestSchema { base_dsn, name }, database)
}

async fn stored_version(database: &DatabaseConnection, key_id: Uuid) -> i32 {
    database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT kek_version FROM key_boxes WHERE id = $1",
            [key_id.into()],
        ))
        .await
        .expect("read extracted KEK version")
        .expect("key box row")
        .try_get("", "kek_version")
        .expect("KEK version column")
}

// NOTE: AI-generated test
#[tokio::test]
async fn extracts_backfills_constrains_and_refuses_kek_version_rollback() {
    dotenvy::dotenv().ok();

    // SAFETY: this integration-test binary has one test and uses deterministic test-only keys.
    unsafe {
        std::env::set_var("OCEANIAM_MASTER_KEY", TEST_MASTER_KEY_HEX);
        std::env::set_var(
            "OCEANIAM_APPLICATION_SECRET_HMAC__KEYS",
            format!("1:{TEST_HMAC_KEY_HEX}"),
        );
        std::env::set_var("MIGRATION_DEFAULT_ROOT_PASSWORD", "migration-test-password");
    }

    let (_schema, database) = isolated_database().await;
    let migrations = Migrator::migrations();
    let target_index = migrations
        .iter()
        .position(|migration| migration.name() == MIGRATION_VERSION)
        .expect("KEK version extraction migration must be registered");
    Migrator::up(&database, Some(target_index as u32))
        .await
        .expect("apply migrations before KEK version extraction");

    let tenant_id = Uuid::now_v7();
    let key_id = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO tenants (id, comment, created_at) VALUES ($1, NULL, now())",
            [tenant_id.into()],
        ))
        .await
        .expect("insert tenant fixture");
    let secret = serde_json::json!({
        "nonce": "AA==",
        "ciphertext": "AA==",
        "key_version": 42,
    });
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO key_boxes (
                id, key_alg, status, created_at, activated_at, retired_at, revoked_at,
                expires_at, secret, tenant_id
             ) VALUES (
                $1, 'rs256', 'active', now(), now(), now() + interval '30 days', NULL,
                now() + interval '60 days', $2, $3
             )",
            vec![key_id.into(), secret.into(), tenant_id.into()],
        ))
        .await
        .expect("insert legacy encrypted key fixture");

    Migrator::up(&database, Some(1))
        .await
        .expect("extract KEK version");
    assert_eq!(stored_version(&database, key_id).await, 42);

    let nullability: String = database
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT is_nullable FROM information_schema.columns
             WHERE table_schema = current_schema()
             AND table_name = 'key_boxes' AND column_name = 'kek_version'",
        ))
        .await
        .expect("read KEK version nullability")
        .expect("KEK version column metadata")
        .try_get("", "is_nullable")
        .expect("decode nullability");
    assert_eq!(nullability, "NO");

    assert!(
        database
            .execute_raw(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE key_boxes SET kek_version = 0 WHERE id = $1",
                [key_id.into()],
            ))
            .await
            .is_err(),
        "zero KEK versions must be rejected"
    );
    assert!(
        database
            .execute_raw(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "UPDATE key_boxes SET kek_version = NULL WHERE id = $1",
                [key_id.into()],
            ))
            .await
            .is_err(),
        "NULL KEK versions must be rejected"
    );

    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "DELETE FROM seaql_migrations WHERE version = $1",
            [MIGRATION_VERSION.into()],
        ))
        .await
        .expect("simulate an unrecorded migration for idempotence check");
    Migrator::up(&database, Some(1))
        .await
        .expect("repeat KEK version extraction idempotently");
    assert_eq!(stored_version(&database, key_id).await, 42);

    // The owner selects backup-only even though SQL could reconstruct the legacy JSON field.
    let error = Migrator::down(&database, Some(1))
        .await
        .expect_err("KEK version extraction is irreversible");
    assert!(error.to_string().contains("irreversible:"));
    assert_eq!(stored_version(&database, key_id).await, 42);

    database.close().await.expect("close migration connection");
}
