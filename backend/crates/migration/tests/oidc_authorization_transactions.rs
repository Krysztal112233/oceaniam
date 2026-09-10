use std::sync::Once;

use migration::{MigrationTrait, Migrator, MigratorTrait};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use sea_orm_migration::SchemaManager;
use uuid::Uuid;

const TEST_MASTER_KEY_HEX: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const TEST_HMAC_KEY_HEX: &str = "89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567";
const MIGRATION_VERSION: &str = "m20260909_044144_create_oidc_authorization_transactions";
const RFC_7636_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
const STATUS_TYPE: &str = "oidc_authorization_transaction_status";
const PKCE_TYPE: &str = "oidc_pkce_method";

static TEST_ENVIRONMENT: Once = Once::new();

fn initialize_test_environment() {
    TEST_ENVIRONMENT.call_once(|| {
        dotenvy::dotenv().ok();

        // SAFETY: every test in this binary calls this Once-guarded initializer before using
        // environment-backed migration or database configuration.
        unsafe {
            std::env::set_var("OCEANIAM_MASTER_KEY", TEST_MASTER_KEY_HEX);
            std::env::set_var("MIGRATION_DEFAULT_ROOT_PASSWORD", "migration-test-password");
            std::env::set_var("OCEANIAM_APPLICATION_SECRET_HMAC__CURRENT_VERSION", "1");
            std::env::set_var(
                "OCEANIAM_APPLICATION_SECRET_HMAC__KEYS__1",
                TEST_HMAC_KEY_HEX,
            );
        }
    });
}

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
    let name = format!("test_oidc_auth_tx_{}", Uuid::now_v7().simple());
    let base = Database::connect(&base_dsn)
        .await
        .expect("connect to migration test database");
    base.execute_raw(Statement::from_string(
        DatabaseBackend::Postgres,
        format!("CREATE SCHEMA {name}"),
    ))
    .await
    .expect("create migration test schema");
    base.close().await.expect("close schema setup connection");

    let database = Database::connect(
        ConnectOptions::new(&base_dsn)
            .set_schema_search_path(format!("{name},public"))
            .to_owned(),
    )
    .await
    .expect("connect to isolated migration test schema");

    (TestSchema { base_dsn, name }, database)
}

fn authorization_transaction_migration() -> Box<dyn MigrationTrait> {
    Migrator::migrations()
        .into_iter()
        .find(|migration| migration.name() == MIGRATION_VERSION)
        .expect("authorization transaction migration must be registered")
}

async fn current_enum_labels(database: &DatabaseConnection, type_name: &str) -> Vec<String> {
    database
        .query_all_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT value.enumlabel \
             FROM pg_type AS candidate \
             JOIN pg_namespace AS namespace ON namespace.oid = candidate.typnamespace \
             JOIN pg_enum AS value ON value.enumtypid = candidate.oid \
             WHERE namespace.nspname = current_schema() AND candidate.typname = $1 \
             ORDER BY value.enumsortorder",
            [type_name.into()],
        ))
        .await
        .expect("query current-schema enum labels")
        .into_iter()
        .map(|row| row.try_get("", "enumlabel").expect("read enum label"))
        .collect()
}

async fn object_state(database: &DatabaseConnection) -> (bool, bool, bool, bool, bool, bool) {
    let row = database
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT \
                 to_regclass(current_schema() || '.oidc_authorization_transactions') IS NOT NULL \
                     AS has_table, \
                 to_regclass(current_schema() || \
                     '.uq_applications_tenant_id_id_oidc_auth_tx') IS NOT NULL \
                     AS has_application_index, \
                 to_regclass(current_schema() || \
                     '.uq_oidc_clients_application_id_id_auth_tx') IS NOT NULL \
                     AS has_client_index, \
                 to_regclass(current_schema() || '.idx_oidc_auth_tx_expires_at') IS NOT NULL \
                     AS has_expiry_index, \
                 to_regtype(current_schema() || \
                     '.oidc_authorization_transaction_status') IS NOT NULL \
                     AS has_status_type, \
                 to_regtype(current_schema() || '.oidc_pkce_method') IS NOT NULL AS has_pkce_type"
                .to_owned(),
        ))
        .await
        .expect("query authorization transaction migration objects")
        .expect("migration object query should return one row");

    (
        row.try_get("", "has_table").expect("read table state"),
        row.try_get("", "has_application_index")
            .expect("read application index state"),
        row.try_get("", "has_client_index")
            .expect("read client index state"),
        row.try_get("", "has_expiry_index")
            .expect("read expiry index state"),
        row.try_get("", "has_status_type")
            .expect("read status type state"),
        row.try_get("", "has_pkce_type")
            .expect("read PKCE type state"),
    )
}

// NOTE: AI-generated test
#[tokio::test]
async fn authorization_transaction_migration_is_incremental_and_reversible() {
    initialize_test_environment();

    let (_schema, database) = isolated_database().await;
    let migrations = Migrator::migrations();
    let target_index = migrations
        .iter()
        .position(|migration| migration.name() == MIGRATION_VERSION)
        .expect("authorization transaction migration must be registered");
    Migrator::up(&database, Some(target_index as u32))
        .await
        .expect("apply migrations preceding authorization transactions");

    let tenant_id = Uuid::now_v7();
    let application_id = Uuid::now_v7();
    let oidc_client_id = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO tenants (id, comment, created_at) VALUES ($1, NULL, now())",
            [tenant_id.into()],
        ))
        .await
        .expect("insert pre-migration tenant");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO applications (id, comment, tenant_id, created_at) \
             VALUES ($1, NULL, $2, now())",
            vec![application_id.into(), tenant_id.into()],
        ))
        .await
        .expect("insert pre-migration application");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO oidc_clients \
             (id, application_id, client_id, name, client_type, application_type, created_at) \
             VALUES ($1, $2, 'incremental-client', 'Incremental client', \
                     'public', 'web', now())",
            vec![oidc_client_id.into(), application_id.into()],
        ))
        .await
        .expect("insert pre-migration OIDC client");

    Migrator::up(&database, Some(1))
        .await
        .expect("apply authorization transaction migration incrementally");
    assert_eq!(
        object_state(&database).await,
        (true, true, true, true, true, true)
    );
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        ["pending", "cancelled"]
    );
    assert_eq!(current_enum_labels(&database, PKCE_TYPE).await, ["s256"]);

    let transaction_id = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO oidc_authorization_transactions \
             (id, tenant_id, application_id, oidc_client_id, issuer, redirect_uri, \
              state, nonce, code_challenge, browser_binding_digest, csrf_digest) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            vec![
                transaction_id.into(),
                tenant_id.into(),
                application_id.into(),
                oidc_client_id.into(),
                "https://issuer.example/oidc/tenant".into(),
                "https://client.example/callback".into(),
                "opaque-state".into(),
                "opaque-nonce".into(),
                RFC_7636_CHALLENGE.into(),
                vec![1_u8; 32].into(),
                vec![2_u8; 32].into(),
            ],
        ))
        .await
        .expect("insert transaction for pre-existing owners");

    let row = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT status::text AS status, revision, terminal_at IS NULL AS no_terminal, \
                    expires_at - created_at = interval '10 minutes' AS exact_lifetime \
             FROM oidc_authorization_transactions WHERE id = $1",
            [transaction_id.into()],
        ))
        .await
        .expect("query inserted transaction")
        .expect("transaction should exist");
    assert_eq!(row.try_get::<String>("", "status").unwrap(), "pending");
    assert_eq!(row.try_get::<i64>("", "revision").unwrap(), 0);
    assert!(row.try_get::<bool>("", "no_terminal").unwrap());
    assert!(row.try_get::<bool>("", "exact_lifetime").unwrap());

    Migrator::down(&database, Some(1))
        .await
        .expect("roll back authorization transaction migration");
    assert_eq!(
        object_state(&database).await,
        (false, false, false, false, false, false)
    );

    let owner_count: i64 = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT COUNT(*)::bigint AS count FROM oidc_clients WHERE id = $1",
            [oidc_client_id.into()],
        ))
        .await
        .expect("query owner after rollback")
        .expect("owner count should return one row")
        .try_get("", "count")
        .expect("read owner count");
    assert_eq!(owner_count, 1, "rollback must preserve existing owners");

    Migrator::up(&database, Some(1))
        .await
        .expect("reapply authorization transaction migration after rollback");
    assert_eq!(
        object_state(&database).await,
        (true, true, true, true, true, true)
    );
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        ["pending", "cancelled"]
    );
    assert_eq!(current_enum_labels(&database, PKCE_TYPE).await, ["s256"]);

    database
        .close()
        .await
        .expect("close migration test database");
}

// NOTE: AI-generated test
#[tokio::test]
async fn authorization_transaction_migration_uses_only_current_schema_enum_types() {
    initialize_test_environment();

    let (_other_schema, other_database) = isolated_database().await;
    other_database
        .execute_unprepared(
            "CREATE TYPE oidc_authorization_transaction_status AS ENUM ('other-status')",
        )
        .await
        .expect("create colliding status type in another schema");
    other_database
        .execute_unprepared("CREATE TYPE oidc_pkce_method AS ENUM ('plain')")
        .await
        .expect("create colliding PKCE type in another schema");

    let (_schema, database) = isolated_database().await;
    let migrations = Migrator::migrations();
    let target_index = migrations
        .iter()
        .position(|migration| migration.name() == MIGRATION_VERSION)
        .expect("authorization transaction migration must be registered");
    Migrator::up(&database, Some(target_index as u32))
        .await
        .expect("apply migrations preceding authorization transactions");

    authorization_transaction_migration()
        .up(&SchemaManager::new(&database))
        .await
        .expect("other-schema enum names must not block the migration");

    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        ["pending", "cancelled"]
    );
    assert_eq!(current_enum_labels(&database, PKCE_TYPE).await, ["s256"]);
    assert_eq!(
        current_enum_labels(&other_database, STATUS_TYPE).await,
        ["other-status"]
    );
    assert_eq!(
        current_enum_labels(&other_database, PKCE_TYPE).await,
        ["plain"]
    );

    database
        .close()
        .await
        .expect("close current-schema migration database");
    other_database
        .close()
        .await
        .expect("close other-schema migration database");
}
