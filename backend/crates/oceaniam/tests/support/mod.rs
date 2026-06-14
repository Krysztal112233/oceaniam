use std::{collections::HashMap, net::SocketAddr, sync::OnceLock};

use migration::{Migrator, MigratorTrait};
use oceaniam::app::{app, build_state};
use oceaniam_common::config::{BackendConfig, CookieConfig, CorsConfig, DatabaseConfig};
use oceaniam_database::{
    helper::{applications::ApplicationHelper, tenants::TenantsHelper},
    model::prelude::{Applications, Tenants},
};
use rand::Rng;
use reqwest::Client;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, Statement, TransactionTrait,
};
use tokio::task::JoinHandle;
use uuid::Uuid;

const DEFAULT_ROOT_PASSWORD_ENV: &str = "MIGRATION_DEFAULT_ROOT_PASSWORD";
static TEST_ROOT_PASSWORD: OnceLock<String> = OnceLock::new();

#[allow(unused)]
pub struct TestApp {
    pub address: String,
    pub client: Client,
    server: JoinHandle<()>,

    /// The isolated schema name. If None, the default schema is used (non-isolated mode).
    pub schema_name: String,

    /// Base DSN (without schema parameter), used for cleaning up the schema.
    pub base_dsn: String,

    /// The root password for the test application.
    /// Populated from `MIGRATION_DEFAULT_ROOT_PASSWORD` env var or auto-generated.
    pub root_password: String,
}

#[allow(unused)]
impl TestApp {
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }

    pub fn dsn_with_schema(&self) -> String {
        add_schema_to_dsn(&self.base_dsn, &self.schema_name)
    }

    pub async fn database(&self) -> DatabaseConnection {
        Database::connect(&self.dsn_with_schema())
            .await
            .expect("failed to connect to test schema database")
    }

    pub async fn seed_tenant_and_application(&self) -> (Uuid, Uuid) {
        let db = self.database().await;

        let tenant_id = Uuid::now_v7();
        Tenants::create_tenant(tenant_id, None::<String>, &db)
            .await
            .unwrap();

        let application_id = Uuid::now_v7();
        Applications::create_application(application_id, tenant_id, &db)
            .await
            .unwrap();

        (tenant_id, application_id)
    }

    pub async fn root_signin(&self) -> String {
        let body = serde_json::json!({
            "name": "root",
            "password": self.root_password,
        });
        let resp: serde_json::Value = self
            .client
            .post(self.url("/auth/tokens"))
            .json(&body)
            .send()
            .await
            .expect("root signin request failed")
            .json()
            .await
            .expect("root signin response parse failed");
        resp["jwt"]
            .as_str()
            .expect("root signin response missing jwt")
            .to_string()
    }

    pub async fn api_create_tenant(&self, token: &str) -> serde_json::Value {
        let body = serde_json::json!({ "comment": null });
        self.client
            .post(self.url("/tenants"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .expect("create tenant request failed")
            .json()
            .await
            .expect("create tenant response parse failed")
    }

    pub async fn api_delete_tenant(&self, token: &str, tenant_id: &str) -> reqwest::Response {
        self.client
            .delete(self.url(&format!("/tenants/{tenant_id}")))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("delete tenant request failed")
    }

    pub async fn api_create_application(&self, token: &str, tenant_id: &str) -> serde_json::Value {
        let body = serde_json::json!({ "comment": null });
        self.client
            .post(self.url(&format!("/tenants/{tenant_id}/applications")))
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .expect("create application request failed")
            .json()
            .await
            .expect("create application response parse failed")
    }

    pub async fn api_delete_application(
        &self,
        token: &str,
        tenant_id: &str,
        application_id: &str,
    ) -> reqwest::Response {
        self.client
            .delete(self.url(&format!(
                "/tenants/{tenant_id}/applications/{application_id}"
            )))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("delete application request failed")
    }

    pub async fn api_create_user(
        &self,
        token: &str,
        tenant_id: &str,
        application_id: &str,
    ) -> serde_json::Value {
        self.api_create_user_with_credentials(
            token,
            tenant_id,
            application_id,
            "test@example.com",
            "TestPassword123!",
        )
        .await
    }

    pub async fn api_create_user_with_credentials(
        &self,
        token: &str,
        tenant_id: &str,
        application_id: &str,
        email: &str,
        password: &str,
    ) -> serde_json::Value {
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });
        self.client
            .post(self.url(&format!(
                "/tenants/{tenant_id}/applications/{application_id}/users"
            )))
            .header("Authorization", format!("Bearer {token}"))
            .json(&body)
            .send()
            .await
            .expect("create user request failed")
            .json()
            .await
            .expect("create user response parse failed")
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Abort the server
        self.server.abort();

        // Run cleanup on a dedicated thread so it does not depend on the test runtime
        // still being alive after `Drop` returns.
        let schema_name = self.schema_name.clone();
        let base_dsn = self.base_dsn.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build schema cleanup runtime");

            runtime.block_on(async move {
                if let Err(e) = drop_schema(&base_dsn, &schema_name).await {
                    eprintln!(
                        "Warning: failed to drop test schema '{}': {}",
                        schema_name, e
                    );
                }
            });
        })
        .join()
        .unwrap();
    }
}

// NOTE: !!!HARD CODED CONFIGURATION!!!
// Deterministic test KEK (32 bytes of zeros in hex) — not secret, tests only.
const TEST_MASTER_KEY_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

fn test_config() -> BackendConfig {
    BackendConfig {
        addr: "0.0.0.0:0".to_owned(),
        database: DatabaseConfig {
            dsn: "postgresql://postgres:postgres@localhost:5432/postgres".to_string(),
            slow_statements_logging_threshold: None,
            max_connections: None,
            min_connections: None,
        },
        cors: CorsConfig {
            allow_origin: "*".to_string(),
        },
        workers: HashMap::new(),
        cookie: CookieConfig::default(),
        master_key: TEST_MASTER_KEY_HEX.to_owned(),
    }
}

/// Creates a test application instance with an isolated database schema.
///
/// Each test that calls this function gets a unique PostgreSQL schema,
/// allowing tests to run concurrently without interfering with each other.
///
/// # Parameters
/// - `base_config`: Base configuration; the database DSN will be modified to use an isolated schema.
///
/// # Returns
/// A `TestApp` instance that automatically cleans up its schema when dropped.
pub async fn spawn_app_with_isolated_schema() -> TestApp {
    let mut test_config = test_config();
    let base_dsn = test_config.database.dsn.clone();

    // Build the DSN with schema
    let schema_name = prepare_isolation_schema(&test_config).await;
    test_config.database.dsn = add_schema_to_dsn(&test_config.database.dsn, &schema_name);

    let root_password = TEST_ROOT_PASSWORD
        .get_or_init(|| {
            match std::env::var(DEFAULT_ROOT_PASSWORD_ENV).ok() {
                Some(password) => password,
                None => {
                    // Generate or retrieve the root password for this test instance
                    let root_password = generate_root_password();

                    // SAFETY: guarded by `TEST_ROOT_PASSWORD`, so concurrent tests cannot race on
                    // this process-wide environment variable while migrations read it.
                    unsafe { std::env::set_var(DEFAULT_ROOT_PASSWORD_ENV, &root_password) };

                    root_password
                }
            }
        })
        .clone();

    // Run migrations in the new schema
    {
        // SAFETY: test-only. Set KEK env var before migration runs so the
        // envelope_encrypt_keys migration can encrypt existing key_boxes rows.
        unsafe {
            std::env::set_var("OCEANIAM__MASTER_KEY", &test_config.master_key);
        }

        let migrate_db = Database::connect(
            ConnectOptions::new(&test_config.database.dsn)
                .set_schema_search_path(format!("{schema_name},public"))
                .to_owned(),
        )
        .await
        .expect("failed to connect to schema for migration");

        let txn = migrate_db
            .begin()
            .await
            .expect("failed to open migration transaction");
        Migrator::up(&txn, None)
            .await
            .expect("failed to run migration");
        txn.commit()
            .await
            .expect("failed to commit test schema migrations");
        migrate_db.close().await.ok();
    }

    // Start the application
    let (server, address) = {
        let state = build_state(&test_config)
            .await
            .expect("failed to build integration test app state");
        let router = app(state, test_config.cors.clone());

        let listener = tokio::net::TcpListener::bind(&test_config.addr)
            .await
            .expect("failed to bind integration test listener");
        let address = listener
            .local_addr()
            .expect("failed to read integration test listener address");

        (
            tokio::spawn(async move {
                axum::serve(listener, router)
                    .await
                    .expect("integration test server exited unexpectedly");
            }),
            address,
        )
    };

    TestApp {
        address: format_address(address),
        client: Client::new(),
        server,
        schema_name,
        base_dsn,
        root_password,
    }
}

/// Gets the root password from environment variable or generates a new one.
fn generate_root_password() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 16;
    let mut rng = rand::thread_rng();

    (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

async fn prepare_isolation_schema(test_config: &BackendConfig) -> String {
    // Generate a unique schema name
    let schema_name = format!("test_schema_{}", Uuid::now_v7().simple());

    // Save the base DSN (without currentSchema parameter) for cleanup
    let base_dsn = test_config.database.dsn.clone();

    // Connect to the default database (usually public schema) to create a new schema
    let root_database = Database::connect(&base_dsn)
        .await
        .expect("failed to connect to database for schema creation");

    // Create the schema
    root_database
        .execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name),
        ))
        .await
        .expect("failed to create test schema");

    // PostgreSQL may acknowledge CREATE SCHEMA before other sessions observe it.
    // Wait until the schema is visible before handing it to the test app.
    let schema_ready = tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
        loop {
            let result = root_database
                .query_one(Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    format!(
                        "SELECT 1 FROM information_schema.schemata WHERE schema_name = '{}'",
                        schema_name
                    ),
                ))
                .await
                .expect("failed to verify test schema visibility");

            if result.is_some() {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    })
    .await;
    assert!(
        schema_ready.is_ok(),
        "timed out waiting for test schema to be ready"
    );

    // Close the admin connection
    root_database.close().await.ok();

    schema_name
}

/// Appends a PostgreSQL search_path override to the DSN.
///
/// `currentSchema` is not honored by the sqlx Postgres connection stack used here,
/// so tests must use the standard libpq `options=-csearch_path=...` form instead.
pub fn add_schema_to_dsn(dsn: &str, schema_name: &str) -> String {
    // Simple string manipulation without using the url crate
    let separator = if dsn.contains('?') { '&' } else { '?' };
    format!(
        "{}{}options=-csearch_path%3D{}",
        dsn, separator, schema_name
    )
}

/// Drops the specified schema
async fn drop_schema(base_dsn: &str, schema_name: &str) -> Result<(), sea_orm::DbErr> {
    let db = Database::connect(base_dsn).await?;

    // Note: We do not terminate other connections because each test uses an isolated schema.
    // Terminating all connections would affect other tests running concurrently.
    // Since the application server has already been aborted, there should be no active connections to this specific schema.

    // Drop the schema (CASCADE will also drop all objects within the schema)
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!("DROP SCHEMA IF EXISTS {} CASCADE", schema_name),
    ))
    .await?;

    db.close().await?;
    Ok(())
}

fn format_address(address: SocketAddr) -> String {
    format!("http://{address}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    // NOTE: AI-generated test
    /// Verifies that the schema created by spawn_app_with_isolated_schema is properly cleaned up when TestApp is dropped.
    #[tokio::test]
    async fn test_isolated_schema_is_dropped_after_test_app_drop() {
        // 1. Create a test app with an isolated schema
        let app = spawn_app_with_isolated_schema().await;

        // 2. Record the schema name
        let schema_name = app.schema_name.clone();
        let base_dsn = app.base_dsn.clone();

        // 3. Verify the schema exists by connecting to it
        let check_dsn = add_schema_to_dsn(&base_dsn, &schema_name);
        let db = Database::connect(&check_dsn)
            .await
            .expect("should connect to schema before drop");

        // Execute a simple query to verify the schema is usable
        let result = db
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT 1 as num".to_string(),
            ))
            .await
            .expect("query should succeed")
            .expect("should get result");
        let value: i32 = result.try_get("", "num").expect("should get column value");
        assert_eq!(value, 1);
        db.close().await.ok();

        // 4. Manually trigger Drop
        drop(app);

        // 5. Wait briefly for async cleanup to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 6. Verify the schema has been dropped by querying information_schema
        let admin_db = Database::connect(&base_dsn)
            .await
            .expect("should connect to admin database");
        let result = admin_db
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT 1 FROM information_schema.schemata WHERE schema_name = '{}'",
                    schema_name
                ),
            ))
            .await
            .expect("query should succeed");
        assert!(
            result.is_none(),
            "schema '{}' should be dropped after TestApp is dropped",
            schema_name
        );
        admin_db.close().await.ok();
    }

    // NOTE: AI-generated test
    /// Verifies that spawn_app_with_isolated_schema produces different schema names.
    #[tokio::test]
    async fn test_isolated_schemas_have_unique_names() {
        let app1 = spawn_app_with_isolated_schema().await;
        let app2 = spawn_app_with_isolated_schema().await;

        let schema1 = app1.schema_name.clone();
        let schema2 = app2.schema_name.clone();

        assert_ne!(
            schema1, schema2,
            "each isolated app should have unique schema name"
        );
    }
}
