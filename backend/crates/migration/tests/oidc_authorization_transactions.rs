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

const A1_MIGRATION_VERSION: &str = "m20260913_094443_alter_oidc_auth_tx_authenticated";
const AUDIT_TYPE: &str = "audit_type";
const SUBJECT_FK: &str = "fk_oidc_auth_tx_subject";

/// Nullable-column and subject-FK state owned by the A1 migration, probed in the connection's
/// current schema.
async fn a1_object_state(database: &DatabaseConnection) -> (bool, bool, bool) {
    let columns = database
        .query_all_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT column_name, is_nullable \
             FROM information_schema.columns \
             WHERE table_schema = current_schema() \
             AND table_name = 'oidc_authorization_transactions' \
             AND column_name IN ('subject_id', 'authenticated_at')"
                .to_owned(),
        ))
        .await
        .expect("query A1 column state");
    let nullable_column = |name: &str| {
        columns.iter().any(|row| {
            row.try_get::<String>("", "column_name")
                .expect("read column name")
                == name
                && row
                    .try_get::<String>("", "is_nullable")
                    .expect("read nullability")
                    == "YES"
        })
    };

    let has_subject_fk = database
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            format!(
                "SELECT EXISTS ( \
                     SELECT 1 FROM pg_constraint \
                     WHERE conname = '{SUBJECT_FK}' \
                     AND conrelid = 'oidc_authorization_transactions'::regclass \
                 ) AS has_fk"
            ),
        ))
        .await
        .expect("query subject FK state")
        .expect("subject FK query should return one row")
        .try_get::<bool>("", "has_fk")
        .expect("read subject FK state");

    (
        nullable_column("subject_id"),
        nullable_column("authenticated_at"),
        has_subject_fk,
    )
}

async fn seed_registration_owner(
    database: &DatabaseConnection,
    client_id: &str,
) -> (Uuid, Uuid, Uuid) {
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
        .expect("insert tenant owner");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO applications (id, comment, tenant_id, created_at) \
             VALUES ($1, NULL, $2, now())",
            vec![application_id.into(), tenant_id.into()],
        ))
        .await
        .expect("insert application owner");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO oidc_clients \
             (id, application_id, client_id, name, client_type, application_type, created_at) \
             VALUES ($1, $2, $3, 'A1 lifecycle client', 'public', 'web', now())",
            vec![
                oidc_client_id.into(),
                application_id.into(),
                client_id.into(),
            ],
        ))
        .await
        .expect("insert OIDC client owner");
    (tenant_id, application_id, oidc_client_id)
}

async fn insert_authorization_transaction(
    database: &DatabaseConnection,
    owner: (Uuid, Uuid, Uuid),
    status: &str,
    subject_id: Option<Uuid>,
) -> Uuid {
    let (tenant_id, application_id, oidc_client_id) = owner;
    let transaction_id = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO oidc_authorization_transactions \
             (id, tenant_id, application_id, oidc_client_id, issuer, redirect_uri, \
              state, nonce, code_challenge, browser_binding_digest, csrf_digest, \
              status, subject_id, authenticated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, \
                     $12::oidc_authorization_transaction_status, $13, \
                     CASE WHEN $13::uuid IS NOT NULL THEN now() END)",
            vec![
                transaction_id.into(),
                tenant_id.into(),
                application_id.into(),
                oidc_client_id.into(),
                "https://issuer.example/oidc/a1".into(),
                "https://client.example/a1/callback".into(),
                "opaque-state".into(),
                "opaque-nonce".into(),
                RFC_7636_CHALLENGE.into(),
                vec![1_u8; 32].into(),
                vec![2_u8; 32].into(),
                status.into(),
                subject_id.into(),
            ],
        ))
        .await
        .expect("insert authorization transaction");
    transaction_id
}

// NOTE: AI-generated test
#[tokio::test]
async fn m20260913_094443_alter_oidc_auth_tx_authenticated_is_incremental_and_reversible() {
    initialize_test_environment();

    let (_schema, database) = isolated_database().await;
    let migrations = Migrator::migrations();
    let target_index = migrations
        .iter()
        .position(|migration| migration.name() == A1_MIGRATION_VERSION)
        .expect("A1 authentication migration must be registered");
    Migrator::up(&database, Some(target_index as u32))
        .await
        .expect("apply migrations preceding the A1 authentication migration");
    assert_eq!(a1_object_state(&database).await, (false, false, false));
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        ["pending", "cancelled"]
    );
    assert!(
        !current_enum_labels(&database, AUDIT_TYPE)
            .await
            .iter()
            .any(|label| label == "oidc_authenticate"),
        "the audit enum must not carry oidc_authenticate before the A1 migration"
    );

    let owner = seed_registration_owner(&database, "a1-incremental-client").await;
    let pending_id = {
        let (tenant_id, application_id, oidc_client_id) = owner;
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
                    "https://issuer.example/oidc/a1".into(),
                    "https://client.example/a1/callback".into(),
                    "opaque-state".into(),
                    "opaque-nonce".into(),
                    RFC_7636_CHALLENGE.into(),
                    vec![1_u8; 32].into(),
                    vec![2_u8; 32].into(),
                ],
            ))
            .await
            .expect("insert pre-A1 pending transaction");
        transaction_id
    };

    Migrator::up(&database, Some(1))
        .await
        .expect("apply the A1 authentication migration incrementally");
    assert_eq!(a1_object_state(&database).await, (true, true, true));
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        ["pending", "cancelled", "authenticated"]
    );
    assert!(
        current_enum_labels(&database, AUDIT_TYPE)
            .await
            .iter()
            .any(|label| label == "oidc_authenticate"),
        "the audit enum must gain oidc_authenticate"
    );

    let subject_id = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO credentials (id, phc) VALUES ($1, 'test-phc')",
            [subject_id.into()],
        ))
        .await
        .expect("insert authenticated subject's credential");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO subjects (id, type, application_id, created_at) \
             VALUES ($1, 'user', $2, now())",
            vec![subject_id.into(), owner.1.into()],
        ))
        .await
        .expect("insert authenticated subject");
    let authenticated_id =
        insert_authorization_transaction(&database, owner, "authenticated", Some(subject_id)).await;
    let stored = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT status::text AS status, subject_id, authenticated_at IS NOT NULL AS stamped \
             FROM oidc_authorization_transactions WHERE id = $1",
            [authenticated_id.into()],
        ))
        .await
        .expect("query authenticated transaction")
        .expect("authenticated transaction should exist");
    assert_eq!(
        stored.try_get::<String>("", "status").unwrap(),
        "authenticated"
    );
    assert_eq!(
        stored.try_get::<Option<Uuid>>("", "subject_id").unwrap(),
        Some(subject_id)
    );
    assert!(stored.try_get::<bool>("", "stamped").unwrap());

    let invalid_subject = database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions SET subject_id = $1 WHERE id = $2",
            vec![Uuid::now_v7().into(), pending_id.into()],
        ))
        .await
        .expect_err("the subject FK must reject a subject that does not exist");
    assert!(
        invalid_subject.to_string().contains(SUBJECT_FK),
        "expected the {SUBJECT_FK} violation, got: {invalid_subject}"
    );

    Migrator::down(&database, Some(1))
        .await
        .expect("roll back the A1 authentication migration");
    assert_eq!(a1_object_state(&database).await, (false, false, false));
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        ["pending", "cancelled", "authenticated"],
        "PostgreSQL cannot drop enum values, so rollback keeps the added labels"
    );
    assert!(
        current_enum_labels(&database, AUDIT_TYPE)
            .await
            .iter()
            .any(|label| label == "oidc_authenticate"),
        "rollback keeps the oidc_authenticate audit label"
    );
    let pending_status: String = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT status::text AS status FROM oidc_authorization_transactions WHERE id = $1",
            [pending_id.into()],
        ))
        .await
        .expect("query pre-A1 transaction after rollback")
        .expect("pre-A1 transaction must survive rollback")
        .try_get("", "status")
        .expect("read pre-A1 transaction status");
    assert_eq!(pending_status, "pending");

    Migrator::up(&database, Some(1))
        .await
        .expect("reapply the A1 authentication migration after rollback");
    assert_eq!(a1_object_state(&database).await, (true, true, true));
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        ["pending", "cancelled", "authenticated"]
    );
    let restored = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT subject_id IS NULL AS no_subject, authenticated_at IS NULL AS no_stamp \
             FROM oidc_authorization_transactions WHERE id = $1",
            [pending_id.into()],
        ))
        .await
        .expect("query pre-A1 transaction after reapply")
        .expect("pre-A1 transaction must survive the up-down-up cycle");
    assert!(restored.try_get::<bool>("", "no_subject").unwrap());
    assert!(restored.try_get::<bool>("", "no_stamp").unwrap());

    let (_fresh_schema, fresh_database) = isolated_database().await;
    Migrator::up(&fresh_database, None)
        .await
        .expect("apply every migration on a fresh schema");
    assert_eq!(
        a1_object_state(&fresh_database).await,
        (true, true, true),
        "a fresh migration run must include the A1 columns and subject FK"
    );
    assert_eq!(
        current_enum_labels(&fresh_database, STATUS_TYPE).await,
        [
            "pending",
            "cancelled",
            "authenticated",
            "awaiting_challenge"
        ]
    );
    assert!(
        current_enum_labels(&fresh_database, AUDIT_TYPE)
            .await
            .iter()
            .any(|label| label == "oidc_authenticate")
    );

    database
        .close()
        .await
        .expect("close incremental migration test database");
    fresh_database
        .close()
        .await
        .expect("close fresh migration test database");
}

// NOTE: AI-generated test
#[tokio::test]
async fn m20260913_094443_alter_oidc_auth_tx_authenticated_creates_the_subject_fk_in_every_schema()
{
    initialize_test_environment();

    let (_first_schema, first_database) = isolated_database().await;
    Migrator::up(&first_database, None)
        .await
        .expect("apply every migration in the first schema");
    let (_second_schema, second_database) = isolated_database().await;
    Migrator::up(&second_database, None)
        .await
        .expect("apply every migration in the second schema");

    for (label, database) in [("first", &first_database), ("second", &second_database)] {
        assert_eq!(
            a1_object_state(database).await,
            (true, true, true),
            "the {label} schema must carry the nullable columns and its own subject FK"
        );
    }

    // Functional enforcement in the second schema: the unqualified pg_constraint probe used to
    // silently skip this schema's FK, so prove an invalid subject is rejected here.
    let owner = seed_registration_owner(&second_database, "a1-second-schema-client").await;
    let pending_id =
        insert_authorization_transaction(&second_database, owner, "pending", None).await;
    let invalid_subject = second_database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions SET subject_id = $1 WHERE id = $2",
            vec![Uuid::now_v7().into(), pending_id.into()],
        ))
        .await
        .expect_err("the second schema's subject FK must reject a subject that does not exist");
    assert!(
        invalid_subject.to_string().contains(SUBJECT_FK),
        "expected the {SUBJECT_FK} violation in the second schema, got: {invalid_subject}"
    );

    // Subject deletion must cascade the authentication snapshot through the second schema's FK.
    let subject_id = Uuid::now_v7();
    second_database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO credentials (id, phc) VALUES ($1, 'test-phc')",
            [subject_id.into()],
        ))
        .await
        .expect("insert second-schema subject's credential");
    second_database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO subjects (id, type, application_id, created_at) \
             VALUES ($1, 'user', $2, now())",
            vec![subject_id.into(), owner.1.into()],
        ))
        .await
        .expect("insert second-schema subject");
    insert_authorization_transaction(&second_database, owner, "authenticated", Some(subject_id))
        .await;
    second_database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "DELETE FROM subjects WHERE id = $1",
            [subject_id.into()],
        ))
        .await
        .expect("delete the authenticated subject");
    let remaining: i64 = second_database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT COUNT(*)::bigint AS count FROM oidc_authorization_transactions \
             WHERE subject_id = $1",
            [subject_id.into()],
        ))
        .await
        .expect("count transactions referencing the deleted subject")
        .expect("reference count should return one row")
        .try_get("", "count")
        .expect("read reference count");
    assert_eq!(
        remaining, 0,
        "deleting a subject must cascade its authentication snapshots in every schema"
    );

    first_database
        .close()
        .await
        .expect("close first-schema migration database");
    second_database
        .close()
        .await
        .expect("close second-schema migration database");
}

const A2_MIGRATION_VERSION: &str = "m20260913_151333_alter_oidc_auth_tx_awaiting_challenge";

/// Nullable `challenge_id` column state owned by the A2 migration, probed in the connection's
/// current schema.
async fn a2_object_state(database: &DatabaseConnection) -> bool {
    database
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT EXISTS ( \
                 SELECT 1 FROM information_schema.columns \
                 WHERE table_schema = current_schema() \
                 AND table_name = 'oidc_authorization_transactions' \
                 AND column_name = 'challenge_id' \
                 AND is_nullable = 'YES' \
             ) AS has_column"
                .to_owned(),
        ))
        .await
        .expect("query A2 column state")
        .expect("A2 column query should return one row")
        .try_get::<bool>("", "has_column")
        .expect("read A2 column state")
}

// NOTE: AI-generated test
#[tokio::test]
async fn m20260913_151333_alter_oidc_auth_tx_awaiting_challenge_is_incremental_and_reversible() {
    initialize_test_environment();

    let (_schema, database) = isolated_database().await;
    let migrations = Migrator::migrations();
    let target_index = migrations
        .iter()
        .position(|migration| migration.name() == A2_MIGRATION_VERSION)
        .expect("A2 challenge migration must be registered");
    Migrator::up(&database, Some(target_index as u32))
        .await
        .expect("apply migrations preceding the A2 challenge migration");
    assert!(!a2_object_state(&database).await);
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        ["pending", "cancelled", "authenticated"]
    );

    let owner = seed_registration_owner(&database, "a2-incremental-client").await;
    let pending_id = insert_authorization_transaction(&database, owner, "pending", None).await;

    Migrator::up(&database, Some(1))
        .await
        .expect("apply the A2 challenge migration incrementally");
    assert!(a2_object_state(&database).await);
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        [
            "pending",
            "cancelled",
            "authenticated",
            "awaiting_challenge"
        ]
    );

    // Functional check: the new enum value and the nullable, FK-free challenge binding are
    // usable together, and the pre-A2 transaction survives untouched.
    let challenge_id = Uuid::now_v7();
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions \
             SET status = 'awaiting_challenge', challenge_id = $1 \
             WHERE id = $2",
            vec![challenge_id.into(), pending_id.into()],
        ))
        .await
        .expect("bind a challenge to the transaction");
    let stored = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT status::text AS status, challenge_id \
             FROM oidc_authorization_transactions WHERE id = $1",
            [pending_id.into()],
        ))
        .await
        .expect("query challenged transaction")
        .expect("challenged transaction should exist");
    assert_eq!(
        stored.try_get::<String>("", "status").unwrap(),
        "awaiting_challenge"
    );
    assert_eq!(
        stored.try_get::<Option<Uuid>>("", "challenge_id").unwrap(),
        Some(challenge_id)
    );

    Migrator::down(&database, Some(1))
        .await
        .expect("roll back the A2 challenge migration");
    assert!(!a2_object_state(&database).await);
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        [
            "pending",
            "cancelled",
            "authenticated",
            "awaiting_challenge"
        ],
        "PostgreSQL cannot drop enum values, so rollback keeps the added label"
    );
    let pending_status: String = database
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT status::text AS status FROM oidc_authorization_transactions WHERE id = $1",
            [pending_id.into()],
        ))
        .await
        .expect("query pre-A2 transaction after rollback")
        .expect("pre-A2 transaction must survive rollback")
        .try_get("", "status")
        .expect("read pre-A2 transaction status");
    assert_eq!(pending_status, "awaiting_challenge");

    Migrator::up(&database, Some(1))
        .await
        .expect("reapply the A2 challenge migration after rollback");
    assert!(a2_object_state(&database).await);
    assert_eq!(
        current_enum_labels(&database, STATUS_TYPE).await,
        [
            "pending",
            "cancelled",
            "authenticated",
            "awaiting_challenge"
        ]
    );

    let (_fresh_schema, fresh_database) = isolated_database().await;
    Migrator::up(&fresh_database, None)
        .await
        .expect("apply every migration on a fresh schema");
    assert!(
        a2_object_state(&fresh_database).await,
        "a fresh migration run must include the A2 challenge column"
    );
    assert_eq!(
        current_enum_labels(&fresh_database, STATUS_TYPE).await,
        [
            "pending",
            "cancelled",
            "authenticated",
            "awaiting_challenge"
        ]
    );

    database
        .close()
        .await
        .expect("close incremental migration test database");
    fresh_database
        .close()
        .await
        .expect("close fresh migration test database");
}
