use std::time::Duration;

use oceaniam_common::{
    config::DatabaseConfig,
    consts,
    error::Error,
};
use oceaniam_database::{
    helper::{applications::ApplicationHelper, tenants::TenantsHelper},
    model::prelude::*,
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tap::Pipe;
use tracing::{error, warn};

pub mod app;
pub mod endpoints;
pub mod middlewares;
pub mod state;

pub async fn setup_database(
    DatabaseConfig {
        dsn,
        slow_statements_logging_threshold,
        max_connections,
        min_connections,
    }: &DatabaseConfig,
) -> Result<DatabaseConnection, Error> {
    let options = ConnectOptions::new(dsn)
        .pipe_borrow_mut(|it| match slow_statements_logging_threshold {
            Some(milis) => it.sqlx_slow_statements_logging_settings(
                log::LevelFilter::Warn,
                Duration::from_micros(*milis),
            ),
            _ => it,
        })
        .pipe_borrow_mut(|it| match max_connections {
            Some(c) => it.max_connections(*c),
            _ => it,
        })
        .pipe_borrow_mut(|it| match min_connections {
            Some(c) => it.min_connections(*c),
            _ => it,
        })
        .pipe_borrow_mut(|it| it.sqlx_logging(false))
        .to_owned();

    let db = Database::connect(options).await.inspect_err(|err| {
        error!(
            %dsn,
            error = %err,
            "failed to connect to database"
        )
    })?;

    init_system(&db).await?;

    Ok(db)
}

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
