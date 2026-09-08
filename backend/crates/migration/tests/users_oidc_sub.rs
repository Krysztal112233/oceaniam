use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use sea_orm_migration::SchemaManager;
use uuid::Uuid;

const TEST_MASTER_KEY_HEX: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const TEST_HMAC_KEY_HEX: &str = "89abcdef0123456789abcdef0123456789abcdef0123456789abcdef01234567";
const MIGRATION_VERSION: &str = "m20260907_174633_add_users_oidc_sub";
const UNIQUE_INDEX: &str = "uq_users_oidc_sub";

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
    let name = format!("test_users_oidc_sub_{}", Uuid::now_v7().simple());
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

async fn insert_application(database: &DatabaseConnection, tenant_id: Uuid) -> Uuid {
    let application_id = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO applications (id, comment, tenant_id, created_at) \
             VALUES ($1, NULL, $2, now())",
            vec![application_id.into(), tenant_id.into()],
        ))
        .await
        .expect("insert application");
    application_id
}

async fn insert_pre_migration_user(
    database: &DatabaseConnection,
    application_id: Uuid,
    user_id: Uuid,
    email: &str,
) {
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO credentials (id, phc) VALUES ($1, 'test-phc')",
            [user_id.into()],
        ))
        .await
        .expect("insert credential");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO subjects (id, type, application_id, created_at) \
             VALUES ($1, 'user', $2, now())",
            vec![user_id.into(), application_id.into()],
        ))
        .await
        .expect("insert subject");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO users (id, application_id, email, phone, nickname, created_at) \
             VALUES ($1, $2, $3, NULL, 'existing', now())",
            vec![user_id.into(), application_id.into(), email.into()],
        ))
        .await
        .expect("insert user");
}

async fn insert_post_migration_user(
    database: &DatabaseConnection,
    application_id: Uuid,
    user_id: Uuid,
    oidc_sub: Uuid,
    email: &str,
) {
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO credentials (id, phc) VALUES ($1, 'test-phc')",
            [user_id.into()],
        ))
        .await
        .expect("insert credential");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO subjects (id, type, application_id, created_at) \
             VALUES ($1, 'user', $2, now())",
            vec![user_id.into(), application_id.into()],
        ))
        .await
        .expect("insert subject");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO users \
             (id, application_id, email, phone, nickname, created_at, oidc_sub) \
             VALUES ($1, $2, $3, NULL, 'new', now(), $4)",
            vec![
                user_id.into(),
                application_id.into(),
                email.into(),
                oidc_sub.into(),
            ],
        ))
        .await
        .expect("insert post-migration user");
}

async fn oidc_sub(database: &DatabaseConnection, user_id: Uuid) -> Uuid {
    database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT oidc_sub FROM users WHERE id = $1",
            [user_id.into()],
        ))
        .await
        .expect("query user")
        .expect("user should exist")
        .try_get("", "oidc_sub")
        .expect("read oidc_sub")
}

// NOTE: AI-generated test
#[tokio::test]
async fn users_oidc_sub_migration_backfills_constrains_is_idempotent_and_rolls_back() {
    dotenvy::dotenv().ok();

    // SAFETY: this integration-test binary has one test, so deterministic test-only
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
        .expect("oidc_sub migration must be registered");
    Migrator::up(&database, Some(target_index as u32))
        .await
        .expect("apply migrations preceding oidc_sub");

    let tenant_id = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO tenants (id, comment, created_at) VALUES ($1, NULL, now())",
            [tenant_id.into()],
        ))
        .await
        .expect("insert tenant");
    let application_a = insert_application(&database, tenant_id).await;
    let application_b = insert_application(&database, tenant_id).await;
    let existing_a = Uuid::now_v7();
    let existing_b = Uuid::now_v7();
    insert_pre_migration_user(&database, application_a, existing_a, "a@example.com").await;
    insert_pre_migration_user(&database, application_b, existing_b, "b@example.com").await;

    database
        .execute_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE users ADD COLUMN oidc_sub UUID".to_owned(),
        ))
        .await
        .expect("add compatible pre-existing oidc_sub column");
    database
        .execute_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("CREATE UNIQUE INDEX {UNIQUE_INDEX} ON users (oidc_sub) WHERE false"),
        ))
        .await
        .expect("create incompatible partial oidc_sub index");
    let incompatible_index = Migrator::up(&database, Some(1))
        .await
        .expect_err("partial oidc_sub index must be rejected");
    assert!(
        incompatible_index
            .to_string()
            .contains("must uniquely cover only users.oidc_sub")
    );
    database
        .execute_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("DROP INDEX {UNIQUE_INDEX}"),
        ))
        .await
        .expect("drop incompatible partial oidc_sub index");

    Migrator::up(&database, Some(1))
        .await
        .expect("apply oidc_sub migration");

    assert_eq!(oidc_sub(&database, existing_a).await, existing_a);
    assert_eq!(oidc_sub(&database, existing_b).await, existing_b);

    let column = database
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT udt_name, is_nullable, column_default \
             FROM information_schema.columns \
             WHERE table_schema = current_schema() AND table_name = 'users' \
             AND column_name = 'oidc_sub'"
                .to_owned(),
        ))
        .await
        .expect("query oidc_sub schema")
        .expect("oidc_sub column should exist");
    assert_eq!(column.try_get::<String>("", "udt_name").unwrap(), "uuid");
    assert_eq!(column.try_get::<String>("", "is_nullable").unwrap(), "NO");
    assert_eq!(
        column
            .try_get::<Option<String>>("", "column_default")
            .unwrap(),
        None
    );

    let index_is_valid: bool = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT i.indisunique AND i.indisvalid AND i.indisready \
                    AND i.indpred IS NULL AND i.indnatts = 1 \
                    AND attribute.attname = 'oidc_sub' AS valid \
             FROM pg_class idx \
             JOIN pg_index i ON i.indexrelid = idx.oid \
             JOIN pg_class users ON users.oid = i.indrelid \
             JOIN pg_namespace namespace ON namespace.oid = users.relnamespace \
             JOIN pg_attribute attribute ON attribute.attrelid = users.oid \
                  AND attribute.attnum = i.indkey[0] \
             WHERE namespace.nspname = current_schema() AND users.relname = 'users' \
                   AND idx.relname = $1",
            [UNIQUE_INDEX.into()],
        ))
        .await
        .expect("query oidc_sub index")
        .expect("oidc_sub index should exist")
        .try_get("", "valid")
        .expect("read index validity");
    assert!(index_is_valid);

    let missing_sub_user = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO credentials (id, phc) VALUES ($1, 'test-phc')",
            [missing_sub_user.into()],
        ))
        .await
        .unwrap();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO subjects (id, type, application_id, created_at) \
             VALUES ($1, 'user', $2, now())",
            vec![missing_sub_user.into(), application_a.into()],
        ))
        .await
        .unwrap();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO users (id, application_id, email, phone, nickname, created_at) \
             VALUES ($1, $2, 'missing@example.com', NULL, 'missing', now())",
            vec![missing_sub_user.into(), application_a.into()],
        ))
        .await
        .expect_err("omitting oidc_sub must fail");

    let new_user = Uuid::now_v7();
    let independent_sub = Uuid::now_v7();
    insert_post_migration_user(
        &database,
        application_a,
        new_user,
        independent_sub,
        "new@example.com",
    )
    .await;

    let duplicate_user = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO credentials (id, phc) VALUES ($1, 'test-phc')",
            [duplicate_user.into()],
        ))
        .await
        .unwrap();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO subjects (id, type, application_id, created_at) \
             VALUES ($1, 'user', $2, now())",
            vec![duplicate_user.into(), application_b.into()],
        ))
        .await
        .unwrap();
    let duplicate = database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO users \
             (id, application_id, email, phone, nickname, created_at, oidc_sub) \
             VALUES ($1, $2, 'duplicate@example.com', NULL, 'duplicate', now(), $3)",
            vec![
                duplicate_user.into(),
                application_b.into(),
                independent_sub.into(),
            ],
        ))
        .await
        .expect_err("duplicate oidc_sub must fail globally");
    assert!(duplicate.to_string().contains("uq_users_oidc_sub"));

    let migration = Migrator::migrations()
        .into_iter()
        .find(|migration| migration.name() == MIGRATION_VERSION)
        .unwrap();
    migration
        .up(&SchemaManager::new(&database))
        .await
        .expect("rerunning migration should be idempotent");
    assert_eq!(oidc_sub(&database, new_user).await, independent_sub);

    Migrator::down(&database, Some(1))
        .await
        .expect("roll back oidc_sub migration");
    let oidc_sub_exists: bool = database
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT EXISTS (SELECT 1 FROM information_schema.columns \
             WHERE table_schema = current_schema() AND table_name = 'users' \
             AND column_name = 'oidc_sub') AS present"
                .to_owned(),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "present")
        .unwrap();
    assert!(!oidc_sub_exists);

    Migrator::up(&database, Some(1))
        .await
        .expect("reapply oidc_sub migration");
    assert_eq!(oidc_sub(&database, existing_a).await, existing_a);
    assert_eq!(
        oidc_sub(&database, new_user).await,
        new_user,
        "destructive down/up intentionally cannot preserve independent subjects"
    );
}
