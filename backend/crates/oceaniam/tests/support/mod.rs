use std::net::SocketAddr;

use migration::{Migrator, MigratorTrait};
use oceaniam::app::{app, build_state};
use oceaniam_common::config::{BackendConfig, CorsConfig, DatabaseConfig};
use rand::Rng;
use reqwest::Client;
use sea_orm::{ConnectionTrait, Database, Statement};
use tokio::task::JoinHandle;
use uuid::Uuid;

const DEFAULT_ROOT_PASSWORD_ENV: &str = "MIGRATION_DEFAULT_ROOT_PASSWORD";

pub struct TestApp {
    pub address: String,
    pub client: Client,
    server: JoinHandle<()>,

    /// The isolated schema name. If None, the default schema is used (non-isolated mode).
    pub schema_name: Option<String>,

    /// Base DSN (without schema parameter), used for cleaning up the schema.
    pub base_dsn: Option<String>,

    /// The root password for the test application.
    /// Populated from `MIGRATION_DEFAULT_ROOT_PASSWORD` env var or auto-generated.
    pub root_password: String,
}

impl TestApp {
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Abort the server
        self.server.abort();

        // Clean up the isolated schema in the background if used
        if let (Some(schema_name), Some(base_dsn)) = (self.schema_name.take(), self.base_dsn.take())
        {
            let schema_name = schema_name.clone();
            tokio::task::spawn(async move {
                if let Err(e) = drop_schema(&base_dsn, &schema_name).await {
                    eprintln!(
                        "Warning: failed to drop test schema '{}': {}",
                        schema_name, e
                    );
                }
            });
        }
    }
}

// NOTE: !!!HARD CODED CONFIGURATION!!!
pub fn test_config() -> BackendConfig {
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
pub async fn spawn_app_with_isolated_schema(mut base_config: BackendConfig) -> TestApp {
    // Generate or retrieve the root password for this test instance
    let root_password = get_or_generate_root_password();

    // Set the environment variable so migrations will use this password
    // SAFETY: This is only called in tests, and each test has its own isolated schema.
    // The environment variable is scoped to the test process.
    unsafe { std::env::set_var(DEFAULT_ROOT_PASSWORD_ENV, &root_password) };

    // Generate a unique schema name
    let schema_name = format!("test_schema_{}", Uuid::new_v4().simple());

    // Save the base DSN (without currentSchema parameter) for cleanup
    let base_dsn = base_config.database.dsn.clone();

    // Connect to the default database (usually public schema) to create a new schema
    let admin_db = Database::connect(&base_dsn)
        .await
        .expect("failed to connect to database for schema creation");

    // Create the schema
    admin_db
        .execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name),
        ))
        .await
        .expect("failed to create test schema");

    // Close the admin connection
    admin_db.close().await.ok();

    // Build the DSN with schema
    let schema_dsn = add_schema_to_dsn(&base_dsn, &schema_name);
    base_config.database.dsn = schema_dsn;

    // Run migrations in the new schema
    let migrate_db = Database::connect(&base_config.database.dsn)
        .await
        .expect("failed to connect to schema for migration");
    Migrator::up(&migrate_db, None)
        .await
        .expect("failed to run migrations on test schema");
    migrate_db.close().await.ok();

    // Start the application
    let state = build_state(&base_config)
        .await
        .expect("failed to build integration test app state");
    let router = app(state, base_config.cors.clone());

    let listener = tokio::net::TcpListener::bind(&base_config.addr)
        .await
        .expect("failed to bind integration test listener");
    let address = listener
        .local_addr()
        .expect("failed to read integration test listener address");

    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("integration test server exited unexpectedly");
    });

    TestApp {
        address: format_address(address),
        client: Client::new(),
        server,
        schema_name: Some(schema_name),
        base_dsn: Some(base_dsn),
        root_password,
    }
}

/// Generates a random password for the root account.
fn gen_password() -> String {
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

/// Gets the root password from environment variable or generates a new one.
fn get_or_generate_root_password() -> String {
    match std::env::var(DEFAULT_ROOT_PASSWORD_ENV) {
        Ok(password) if password.len() > 6 => password,
        _ => gen_password(),
    }
}

/// Appends the currentSchema parameter to the DSN
fn add_schema_to_dsn(dsn: &str, schema: &str) -> String {
    // Simple string manipulation without using the url crate
    let separator = if dsn.contains('?') { '&' } else { '?' };
    format!("{}{}currentSchema={}", dsn, separator, schema)
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
        let app = spawn_app_with_isolated_schema(test_config()).await;

        // 2. Record the schema name
        let schema_name = app.schema_name.clone().expect("schema_name should exist");
        let base_dsn = app.base_dsn.clone().expect("base_dsn should exist");

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
        let app1 = spawn_app_with_isolated_schema(test_config()).await;
        let app2 = spawn_app_with_isolated_schema(test_config()).await;

        let schema1 = app1.schema_name.as_ref().unwrap();
        let schema2 = app2.schema_name.as_ref().unwrap();

        assert_ne!(
            schema1, schema2,
            "each isolated app should have unique schema name"
        );
    }
}
