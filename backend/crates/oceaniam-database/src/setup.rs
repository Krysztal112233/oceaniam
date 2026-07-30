use std::time::Duration;

use oceaniam_common::{config::DatabaseConfig, consts};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tap::Pipe;
use tracing::{error, warn};

use crate::error::Error;
use crate::helper::{applications::ApplicationHelper, tenants::TenantsHelper};
use crate::model::prelude::*;

/// Create a database connection from the given configuration.
///
/// Applies pool size and slow-query logging settings before connecting.
#[tracing::instrument(
    level = "info",
    name = "db.connect",
    skip_all,
    fields(otel.kind = "internal")
)]
pub async fn connect(config: &DatabaseConfig) -> Result<DatabaseConnection, Error> {
    let options = ConnectOptions::new(&config.dsn)
        .pipe_borrow_mut(|it| match config.slow_statements_logging_threshold {
            Some(milis) => it.sqlx_slow_statements_logging_settings(
                log::LevelFilter::Warn,
                Duration::from_micros(milis),
            ),
            _ => it,
        })
        .pipe_borrow_mut(|it| match config.max_connections {
            Some(c) => it.max_connections(c),
            _ => it,
        })
        .pipe_borrow_mut(|it| match config.min_connections {
            Some(c) => it.min_connections(c),
            _ => it,
        })
        .pipe_borrow_mut(|it| it.sqlx_logging(false))
        .to_owned();

    Ok(Database::connect(options).await.inspect_err(|err| {
        error!(
            dsn = %config.dsn,
            error = %err,
            "failed to connect to database"
        )
    })?)
}

/// Ensure system tenant and system application exist.
///
/// Called once at startup by both the backend and worker processes.
#[tracing::instrument(
    level = "info",
    name = "db.init_system",
    skip_all,
    fields(otel.kind = "internal")
)]
pub async fn init_system(db: &DatabaseConnection) -> Result<(), Error> {
    if !Tenants::is_system_tenant_exist(db).await? {
        warn!(
            tenant_id = %consts::SYSTEM_TENANT_UUID,
            "system tenant missing; creating builtin tenant"
        );

        Tenants::create_tenant(
            consts::SYSTEM_TENANT_UUID,
            Some("System builtin tenant"),
            db,
        )
        .await?;
    }

    if !Applications::is_system_application_exist(db).await? {
        warn!(
            tenant_id = %consts::SYSTEM_TENANT_UUID,
            application_id = %consts::SYSTEM_APPLICATION_UUID,
            "system application missing; creating builtin application"
        );
        Applications::create_application(
            consts::SYSTEM_APPLICATION_UUID,
            consts::SYSTEM_TENANT_UUID,
            db,
        )
        .await?;
    }

    Ok(())
}
