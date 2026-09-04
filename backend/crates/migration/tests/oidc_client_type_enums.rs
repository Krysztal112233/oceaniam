use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use uuid::Uuid;

const TEST_MASTER_KEY_HEX: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const TEST_HMAC_KEY_HEX: &str = "89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567";
const MIGRATION_VERSION: &str = "m20260904_031809_alter_oidc_client_types_to_enums";

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
    let name = format!("test_oidc_client_enum_{}", Uuid::now_v7().simple());
    let base = Database::connect(&base_dsn).await.unwrap_or_else(|error| {
        panic!(
            "failed to connect to the migration test database; set \
             OCEANIAM_TEST_DATABASE_DSN, OCEANIAM_DATABASE__DSN, or DATABASE_URL: {error}"
        )
    });
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

async fn column_type(database: &DatabaseConnection, column_name: &str) -> (String, Option<i32>) {
    let row = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT udt_name, character_maximum_length::integer AS max_length \
             FROM information_schema.columns \
             WHERE table_schema = current_schema() \
               AND table_name = 'oidc_clients' \
               AND column_name = $1",
            [column_name.into()],
        ))
        .await
        .expect("query OIDC client column type")
        .expect("OIDC client column should exist");

    (
        row.try_get("", "udt_name").expect("read udt_name"),
        row.try_get("", "max_length").expect("read max_length"),
    )
}

async fn enum_labels(database: &DatabaseConnection) -> Vec<(String, String)> {
    database
        .query_all_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT type.typname, value.enumlabel \
             FROM pg_type AS type \
             JOIN pg_enum AS value ON value.enumtypid = type.oid \
             JOIN pg_namespace AS namespace ON namespace.oid = type.typnamespace \
             WHERE namespace.nspname = current_schema() \
               AND type.typname IN ('oidc_client_type', 'oidc_application_type') \
             ORDER BY type.typname, value.enumsortorder"
                .to_owned(),
        ))
        .await
        .expect("query OIDC enum labels")
        .into_iter()
        .map(|row| {
            (
                row.try_get("", "typname").expect("read enum type name"),
                row.try_get("", "enumlabel").expect("read enum label"),
            )
        })
        .collect()
}

// NOTE: AI-generated test
#[tokio::test]
async fn oidc_client_enum_migration_preserves_existing_rows_and_rolls_back() {
    dotenvy::dotenv().ok();

    // SAFETY: this integration-test binary has one test, so these deterministic test-only
    // environment values cannot race another test in this process.
    unsafe {
        std::env::set_var("OCEANIAM_MASTER_KEY", TEST_MASTER_KEY_HEX);
        std::env::set_var("MIGRATION_DEFAULT_ROOT_PASSWORD", "migration-test-password");
        std::env::set_var("OCEANIAM_APPLICATION_SECRET_HMAC__CURRENT_VERSION", "1");
        std::env::set_var(
            "OCEANIAM_APPLICATION_SECRET_HMAC__KEYS__1",
            TEST_HMAC_KEY_HEX,
        );
    }

    let (_schema, database) = isolated_database().await;
    let migrations = Migrator::migrations();
    let target_index = migrations
        .iter()
        .position(|migration| migration.name() == MIGRATION_VERSION)
        .expect("OIDC client enum migration must be registered");
    Migrator::up(&database, Some(target_index as u32))
        .await
        .expect("apply migrations preceding the OIDC enum conversion");

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
        .expect("insert tenant");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO applications (id, comment, tenant_id) VALUES ($1, NULL, $2)",
            vec![application_id.into(), tenant_id.into()],
        ))
        .await
        .expect("insert application");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO oidc_clients \
             (id, application_id, client_id, name, client_type, application_type) \
             VALUES ($1, $2, 'existing-client', 'Existing client', 'public', 'web')",
            vec![oidc_client_id.into(), application_id.into()],
        ))
        .await
        .expect("insert pre-migration OIDC client");

    assert_eq!(
        column_type(&database, "client_type").await,
        ("varchar".to_owned(), Some(16))
    );
    assert_eq!(
        column_type(&database, "application_type").await,
        ("varchar".to_owned(), Some(16))
    );

    Migrator::up(&database, Some(1))
        .await
        .expect("apply OIDC client enum migration");

    assert_eq!(
        column_type(&database, "client_type").await,
        ("oidc_client_type".to_owned(), None)
    );
    assert_eq!(
        column_type(&database, "application_type").await,
        ("oidc_application_type".to_owned(), None)
    );
    assert_eq!(
        enum_labels(&database).await,
        vec![
            ("oidc_application_type".to_owned(), "web".to_owned()),
            ("oidc_client_type".to_owned(), "public".to_owned()),
        ]
    );

    let row = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT client_type::text AS client_type, \
                    application_type::text AS application_type \
             FROM oidc_clients WHERE id = $1",
            [oidc_client_id.into()],
        ))
        .await
        .expect("query converted OIDC client")
        .expect("converted OIDC client should remain");
    assert_eq!(
        row.try_get::<String>("", "client_type")
            .expect("read converted client_type"),
        "public"
    );
    assert_eq!(
        row.try_get::<String>("", "application_type")
            .expect("read converted application_type"),
        "web"
    );

    Migrator::down(&database, Some(1))
        .await
        .expect("roll back OIDC client enum migration");

    assert_eq!(
        column_type(&database, "client_type").await,
        ("varchar".to_owned(), Some(16))
    );
    assert_eq!(
        column_type(&database, "application_type").await,
        ("varchar".to_owned(), Some(16))
    );
    assert!(
        enum_labels(&database).await.is_empty(),
        "rollback should remove both OIDC enum types"
    );

    let checks: Vec<String> = database
        .query_all_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT constraint_name \
             FROM information_schema.table_constraints \
             WHERE table_schema = current_schema() \
               AND table_name = 'oidc_clients' \
               AND constraint_type = 'CHECK' \
               AND constraint_name IN ( \
                   'oidc_clients_client_type_check', \
                   'oidc_clients_application_type_check' \
               ) \
             ORDER BY constraint_name"
                .to_owned(),
        ))
        .await
        .expect("query restored OIDC check constraints")
        .into_iter()
        .map(|row| {
            row.try_get("", "constraint_name")
                .expect("read constraint name")
        })
        .collect();
    assert_eq!(
        checks,
        vec![
            "oidc_clients_application_type_check".to_owned(),
            "oidc_clients_client_type_check".to_owned(),
        ]
    );

    let row = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT client_type, application_type FROM oidc_clients WHERE id = $1",
            [oidc_client_id.into()],
        ))
        .await
        .expect("query rolled-back OIDC client")
        .expect("rolled-back OIDC client should remain");
    assert_eq!(
        row.try_get::<String>("", "client_type")
            .expect("read rolled-back client_type"),
        "public"
    );
    assert_eq!(
        row.try_get::<String>("", "application_type")
            .expect("read rolled-back application_type"),
        "web"
    );

    Migrator::up(&database, Some(1))
        .await
        .expect("reapply OIDC client enum migration after rollback");
    database
        .close()
        .await
        .expect("close migration test database");
}
